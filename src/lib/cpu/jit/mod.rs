use super::{decode, opcode, CpuState};
use crate::cpu::bus::{BusDevice, SizedReadResult};
use inkwell::values::AnyValue;
use std::{collections, rc::Rc};

#[cfg(test)]
pub use super::test::harness;

mod branch;
mod immed;
mod jump;
mod mem;
mod mult;
mod register_ops;

type BusType = crate::cpu::bus_vec::VecBus;
type TbDynFunc<'ctx> =
    unsafe extern "C" fn(state: *mut CpuState, bus: *mut BusType, mgr: *mut TbManager<'ctx>);

struct DelaySlotArg<'ctx> {
    count: u64,
    immed: u16,
    value: inkwell::values::BasicValueEnum<'ctx>,
}

pub struct TranslationBlock<'ctx> {
    id: u64,
    count_uniq: u64,
    finalized: bool,

    ctx: &'ctx inkwell::context::Context,
    module: inkwell::module::Module<'ctx>,
    ee: inkwell::execution_engine::ExecutionEngine<'ctx>,
    builder: inkwell::builder::Builder<'ctx>,

    func: inkwell::values::FunctionValue<'ctx>,
    //func_block: inkwell::basic_block::BasicBlock<'ctx>,
    state_arg: inkwell::values::PointerValue<'ctx>,
    bus_arg: inkwell::values::PointerValue<'ctx>,
    mgr_arg: inkwell::values::PointerValue<'ctx>,

    delay_slot_hazard: Option<fn(&mut TranslationBlock)>,
    delay_slot_arg: Option<DelaySlotArg<'ctx>>,

    tb_func: Option<inkwell::execution_engine::JitFunction<'ctx, TbDynFunc<'ctx>>>,
}

pub(crate) struct TbManager<'ctx> {
    trie: super::trie::Trie<TranslationBlock<'ctx>>,
}

fn new_tb<'ctx>(
    id: u64,
    ctx: &'ctx inkwell::context::Context,
) -> Result<TranslationBlock<'ctx>, String> {
    let module = ctx.create_module(&format!("tb_mod_{}", id));
    let ee = module
        .create_jit_execution_engine(inkwell::OptimizationLevel::Less)
        .map_err(|e| e.to_string())?;
    let builder = ctx.create_builder();

    let i32_type = ctx.i32_type();
    let i8_type = ctx.i8_type();
    let mips_state_type = ctx.opaque_struct_type("mips_state");
    mips_state_type.set_body(&[i32_type.into(); 36], false);

    let bus_type = ctx
        .opaque_struct_type("mips_bus")
        .ptr_type(inkwell::AddressSpace::Generic);
    let tb_mgr_type = ctx
        .opaque_struct_type("tb_manager")
        .ptr_type(inkwell::AddressSpace::Generic);
    let bool_type = ctx.bool_type();

    let state_type = module
        .get_struct_type("mips_state")
        .unwrap()
        .ptr_type(inkwell::AddressSpace::Generic);

    let read_fn = module.add_function(
        "tb_mem_read",
        bool_type.fn_type(
            &[
                bus_type.into(),
                tb_mgr_type.into(),
                state_type.ptr_type(inkwell::AddressSpace::Generic).into(),
                i32_type.into(),
                i32_type.into(),
                i8_type.into(),
                bool_type.into(),
            ],
            false,
        ),
        None,
    );
    let write_fn = module.add_function(
        "tb_mem_write",
        bool_type.fn_type(
            &[
                bus_type.into(),
                tb_mgr_type.into(),
                i32_type.into(),
                i32_type.into(),
                i32_type.into(),
            ],
            false,
        ),
        None,
    );

    ee.add_global_mapping(&read_fn, tb_mem_read as usize);
    ee.add_global_mapping(&write_fn, tb_mem_write as usize);

    let void_type = ctx.void_type();
    let fn_type = void_type.fn_type(
        &[state_type.into(), bus_type.into(), tb_mgr_type.into()],
        false,
    );
    let func_name = format!("tb_func_{}", id);
    let func = module.add_function(&func_name, fn_type, None);

    let state_arg = func
        .get_nth_param(0)
        .ok_or("No state arg")?
        .into_pointer_value();
    let bus_arg = func
        .get_nth_param(1)
        .ok_or("No bus arg")?
        .into_pointer_value();
    let mgr_arg = func
        .get_nth_param(2)
        .ok_or("No manager arg")?
        .into_pointer_value();

    let block = ctx.append_basic_block(func, &func_name);
    builder.position_at_end(block);

    Ok(TranslationBlock {
        id,
        count_uniq: 0,
        finalized: false,
        ctx,
        module,
        ee,
        builder,
        func,
        //func_block: block,
        state_arg,
        bus_arg,
        mgr_arg,
        delay_slot_hazard: None,
        delay_slot_arg: None,
        tb_func: None,
    })
}

impl<'ctx> TbManager<'ctx> {
    pub fn new() -> Self {
        Self {
            trie: super::trie::Trie::default(),
        }
    }

    pub fn get_tb(
        &mut self,
        ctx: &'ctx inkwell::context::Context,
        addr: u32,
        bus: &mut impl BusDevice,
    ) -> Result<Rc<TranslationBlock<'ctx>>, String> {
        if let Some(tb) = self.trie.lookup(addr) {
            return Ok(tb.clone());
        }

        let mut tb = new_tb(addr as u64, ctx)?;
        tb.translate(bus, addr)?;
        tb.finalize();
        let tb_rc = Rc::new(tb);
        self.trie.insert(addr, &tb_rc)?;
        return Ok(tb_rc);
    }

    fn invalidate(&mut self, addr: u32) {
        self.trie.invalidate(addr);
    }
}

impl<'ctx> TranslationBlock<'ctx> {
    // Should be called whenever an instruction has finished translating itself.
    // Any delay slot hazards introduced by that function should only be pushed
    // after calling this function
    fn instr_finished_emitting(&mut self) {
        if self.finalized {
            return;
        }

        if let Some(f) = self.delay_slot_hazard {
            f(self);
            self.delay_slot_hazard = None;
            self.delay_slot_arg = None;
        }

        self.count_uniq += 1;
    }

    fn gep_gp_register(&self, reg: u8, name: &str) -> inkwell::values::PointerValue<'ctx> {
        assert!(reg < 32);
        assert!(reg > 0);
        self.builder
            .build_struct_gep(self.state_arg, (reg as u32) - 1, name)
            .unwrap()
    }

    fn gep_hi(&self, prefix: &str) -> inkwell::values::PointerValue<'ctx> {
        self.builder
            .build_struct_gep(self.state_arg, 31, &format!("{}_hi", prefix))
            .unwrap()
    }

    fn gep_lo(&self, prefix: &str) -> inkwell::values::PointerValue<'ctx> {
        self.builder
            .build_struct_gep(self.state_arg, 32, &format!("{}_lo", prefix))
            .unwrap()
    }

    fn gep_pc(&self, prefix: &str) -> inkwell::values::PointerValue<'ctx> {
        self.builder
            .build_struct_gep(self.state_arg, 33, &format!("{}_pc", prefix))
            .unwrap()
    }

    fn apply_load_delay_if_present(&mut self) {
        let i32_type = self.ctx.i32_type();
        let i64_type = self.ctx.i64_type();

        let register = self
            .builder
            .build_struct_gep(self.state_arg, 34, "ld_delay_reg")
            .unwrap();

        // FIXME: Assert that register_val is in [0, 31]
        let register_val = self.builder.build_load(register, "ld_delay_reg_val");
        let reg_delay_apply_cond = self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            register_val.into_int_value(),
            i32_type.const_zero(),
            "ld_delay_in_use",
        );

        // If the delay slot is not in use, use the register_val as offset so it's a valid index
        // into the array
        let reg_state_offset = self.builder.build_int_sub(
            register_val.into_int_value(),
            i32_type.const_int(1, false),
            "ld_delay_reg_state_offset",
        );
        let reg_adjusted = self
            .builder
            .build_select(
                reg_delay_apply_cond,
                reg_state_offset,
                register_val.into_int_value(),
                "ld_delay_reg_adjusted",
            )
            .into_int_value();

        // This causes LLVM to segfault for some reason?
        /*
        let reg_ptr = unsafe {
            self.builder.build_in_bounds_gep(
                self.state_arg,
                &[i32_type.const_zero(), reg_adjusted],
                "ld_delay_reg_ptr",
            )
        };
        */

        let reg_ptr_offset = self
            .builder
            .build_int_mul(reg_adjusted, i64_type.const_int(4, true), "fsfsds")
            .const_z_ext(i64_type);
        let state_int = self
            .builder
            .build_ptr_to_int(self.state_arg, i64_type, "ld_state_ptr_int");
        let reg_ptr_int =
            self.builder
                .build_int_add(state_int, reg_ptr_offset, "ld_state_reg_ptr_int");
        let reg_ptr = self.builder.build_int_to_ptr(
            reg_ptr_int,
            i32_type.ptr_type(inkwell::AddressSpace::Generic),
            "ld_state_reg_ptr",
        );

        let load_value_ptr = self
            .builder
            .build_struct_gep(self.state_arg, 35, "ld_delay_value")
            .unwrap();

        // Select the delay value if the delay reg is set (within [1, 31]), otherwise reload the
        // same value that's currently in the register
        // FIXME: When the TB prologue uses a branching version of this, we can remove this select 
        // since this path will only be invoked by a delay slot action when the load register is
        // set.
        let reg_ptr_select = self
            .builder
            .build_select(
                reg_delay_apply_cond,
                load_value_ptr,
                reg_ptr,
                "ld_delay_select_ptr",
            )
            .into_pointer_value();

        let reg_new_val = self
            .builder
            .build_load(reg_ptr_select, "ld_delay_new_reg_val");
        self.builder.build_store(reg_ptr, reg_new_val);
        self.builder.build_store(register, i32_type.const_zero());
    }

    fn mem_read(
        &self,
        addr: inkwell::values::BasicMetadataValueEnum<'ctx>,
        size: inkwell::values::BasicMetadataValueEnum<'ctx>,
        reg: inkwell::values::BasicMetadataValueEnum<'ctx>,
        sign_extend: inkwell::values::BasicMetadataValueEnum<'ctx>,
        name: &str,
    ) -> inkwell::values::BasicValueEnum<'ctx> {
        let read_fn = self.module.get_function("tb_mem_read").unwrap();
        self.builder
            .build_call(
                read_fn,
                &[
                    self.bus_arg.into(),
                    self.mgr_arg.into(),
                    self.state_arg.into(),
                    addr,
                    size,
                    reg,
                    sign_extend,
                ],
                name,
            )
            .try_as_basic_value()
            .left()
            .unwrap()
    }

    fn mem_write(
        &self,
        addr: inkwell::values::BasicMetadataValueEnum<'ctx>,
        size: inkwell::values::BasicMetadataValueEnum<'ctx>,
        value: inkwell::values::BasicMetadataValueEnum<'ctx>,
        name: &str,
    ) -> inkwell::values::BasicValueEnum<'ctx> {
        let write_fn = self.module.get_function("tb_mem_write").unwrap();
        self.builder
            .build_call(
                write_fn,
                &[self.bus_arg.into(), self.mgr_arg.into(), addr, size, value],
                name,
            )
            .try_as_basic_value()
            .left()
            .unwrap()
    }

    fn get_gpr_value(&self, reg: u8, prefix: &str) -> inkwell::values::IntValue<'ctx> {
        let i32_type = self.ctx.i32_type();
        if reg == 0 {
            i32_type.const_zero().into()
        } else {
            let ptr = self.gep_gp_register(reg, &format!("{}_src_ptr", prefix));
            self.builder
                .build_load(ptr, &format!("{}_src_reg", prefix))
                .into_int_value()
        }
    }

    fn emit_r_instr(&mut self, instr: &decode::MipsRInstr) {
        match instr.function {
            opcode::MipsFunction::Sll => self.emit_sll(instr),
            opcode::MipsFunction::Sllv => self.emit_sllv(instr),
            opcode::MipsFunction::Srl => self.emit_srl(instr),
            opcode::MipsFunction::Sra => self.emit_sra(instr),
            opcode::MipsFunction::Slrv => self.emit_slrv(instr),
            opcode::MipsFunction::Srav => self.emit_srav(instr),
            opcode::MipsFunction::Jr => self.emit_jr(instr),
            opcode::MipsFunction::Jalr => self.emit_jalr(instr),
            opcode::MipsFunction::Add => self.emit_add(instr),
            opcode::MipsFunction::AddU => self.emit_addu(instr),
            opcode::MipsFunction::Sub => self.emit_sub(instr),
            opcode::MipsFunction::Subu => self.emit_subu(instr),
            opcode::MipsFunction::Mflo => self.emit_mflo(instr),
            opcode::MipsFunction::Mfhi => self.emit_mfhi(instr),
            opcode::MipsFunction::Mtlo => self.emit_mtlo(instr),
            opcode::MipsFunction::Mthi => self.emit_mthi(instr),
            opcode::MipsFunction::DivU => self.emit_divu(instr),
            opcode::MipsFunction::Mult => self.emit_mult(instr),
            opcode::MipsFunction::MultU => self.emit_multu(instr),
            opcode::MipsFunction::Or => self.emit_or(instr),
            opcode::MipsFunction::Nor => self.emit_nor(instr),
            opcode::MipsFunction::Xor => self.emit_xor(instr),
            opcode::MipsFunction::And => self.emit_and(instr),
            opcode::MipsFunction::Sltu => self.emit_sltu(instr),
            opcode::MipsFunction::Slt => self.emit_slt(instr),
            _ => panic!("Not implemented: {}", instr.function),
        }
    }

    fn emit_special_branch(&mut self, instr: &decode::MipsIInstr) {
        let special_op =
            num::FromPrimitive::from_u8(instr.t_reg).unwrap_or(opcode::MipsBranchSpecial::Invalid);
        match special_op {
            _ => panic!("Not implemented: {}", special_op),
        }
    }

    fn emit_i_instr(&mut self, instr: &decode::MipsIInstr) {
        match instr.opcode {
            opcode::MipsOpcode::RegisterImm => self.emit_special_branch(instr),
            opcode::MipsOpcode::Beq => self.emit_beq(instr),
            opcode::MipsOpcode::Bgtz => self.emit_bgtz(instr),
            opcode::MipsOpcode::Bne => self.emit_bne(instr),
            opcode::MipsOpcode::AddIU => self.emit_addiu(instr),
            opcode::MipsOpcode::OrI => self.emit_ori(instr),
            opcode::MipsOpcode::Lui => self.emit_lui(instr),
            opcode::MipsOpcode::Lb => self.emit_lb(instr),
            opcode::MipsOpcode::Lbu => self.emit_lbu(instr),
            opcode::MipsOpcode::Lw => self.emit_lw(instr),
            opcode::MipsOpcode::Sb => self.emit_sb(instr),
            opcode::MipsOpcode::Sw => self.emit_sw(instr),
            opcode::MipsOpcode::SltI => self.emit_slti(instr),
            opcode::MipsOpcode::SltIU => self.emit_sltiu(instr),
            _ => panic!("Not implemented: {}", instr.opcode),
        }
    }

    fn emit_j_instr(&mut self, instr: &decode::MipsJInstr) {
        match instr.opcode {
            opcode::MipsOpcode::J => self.emit_j(instr),
            opcode::MipsOpcode::Jal => self.emit_jal(instr),
            _ => panic!("Not J type instruction: {}", instr.opcode),
        }
    }

    pub fn translate(&mut self, bus: &mut dyn BusDevice, pc: u32) -> Result<u32, String> {
        // FIXME: Use separate branches for initial load delay application to improve performance
        self.apply_load_delay_if_present();

        let mut addr = pc;
        while !self.finalized {
            let read_result = bus.read(addr, 32).map_err(|_| "Failed to read instr")?;
            if let SizedReadResult::Dword(instr_raw) = read_result {
                let instr = super::decode::mips_decode(instr_raw);

                match instr {
                    decode::MipsInstr::RType(r) => self.emit_r_instr(&r),
                    decode::MipsInstr::IType(i) => self.emit_i_instr(&i),
                    decode::MipsInstr::JType(j) => self.emit_j_instr(&j),
                    _ => {
                        self.emit_r_instr(&decode::MipsRInstr {
                            s_reg: 0,
                            t_reg: 0,
                            d_reg: 0,
                            shamt: 0,
                            function: opcode::MipsFunction::Sll,
                        });
                        //return Err(format!("Invalid instruction {:#08x}: {:#08x} {}", addr, instr_raw, instr));
                    }
                }

                addr += 4;
                if ((addr >> 2) & 0x3f == 0) && !self.finalized {
                    let i32_type = self.ctx.i32_type();
                    let pc_val = i32_type.const_int(addr as u64, false);
                    let pc_ptr = self.gep_pc("block_end");
                    self.builder.build_store(pc_ptr, pc_val);
                    self.builder.build_return(None);
                    self.finalized = true;
                }
            } else {
                panic!(
                    "Read of size 32 didn't return a dword? Instead have {:?}",
                    read_result
                );
            }
        }

        // Do not allow a delay action on the final instruction of the block, all must be
        // consumed
        assert!(self.delay_slot_hazard.is_none());
        self.builder.build_return(None);

        Ok(addr)
    }

    pub fn finalize(&mut self) {
        unsafe { self.tb_func = self.ee.get_function(&format!("tb_func_{}", self.id)).ok() }
    }

    pub(crate) fn execute(
        &self,
        state: &mut CpuState,
        bus: &mut BusType,
        tb_mgr: &mut TbManager<'ctx>,
    ) -> Result<(), String> {
        if let Some(func) = self.tb_func.as_ref() {
            unsafe {
                func.call(state, bus, tb_mgr);
                Ok(())
            }
        } else {
            Err(String::from("Failed to compile TB"))
        }
    }
}

impl<'ctx> std::fmt::Display for TranslationBlock<'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.func.print_to_string().fmt(f)
    }
}

#[no_mangle]
pub(crate) unsafe extern "C" fn tb_mem_read(
    bus: *mut BusType,
    _mgr: *mut TbManager,
    state: *mut CpuState,
    addr: u32,
    size: u32,
    reg: u8,
    sign_extend: bool,
) -> bool {
    match (*bus).read(addr, size) {
        Ok(v) => {
            (*state).load_delay_register_value = match v {
                SizedReadResult::Byte(b) => {
                    if sign_extend {
                        b as i8 as u32
                    } else {
                        b as u32
                    }
                }
                SizedReadResult::Word(w) => {
                    if sign_extend {
                        w as i16 as u32
                    } else {
                        w as u32
                    }
                }
                SizedReadResult::Dword(d) => d,
            };

            (*state).load_delay_register = reg as u32;

            true
        }
        Err(e) => {
            panic!("tb_mem_read err: {:#08x?}", e);
        }
    }
}

#[no_mangle]
pub(crate) unsafe extern "C" fn tb_mem_write(
    bus: *mut BusType,
    mgr: *mut TbManager,
    addr: u32,
    size: u32,
    value: u32,
) -> bool {
    let wv = (*bus).write(addr, size, value);
    (*mgr).invalidate(addr);
    if let Err(e) = wv {
        panic!("tb_mem_write err: {:#08x?}", e);
    }
    match wv {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn execute(bus: &mut BusType, state: &mut CpuState) -> Result<(), String> {
    let ctx = inkwell::context::Context::create();
    let mut tb_mgr = TbManager::new();
    let mut prev_pc = 0;
    let mut icount_tot = 0;
    let mut icount = 0;
    let now = std::time::Instant::now();
    let mut prev_elapsed: u128 = 0;

    let mut mips_avg: f64 = 0.0;
    let mut mips_min: f64 = f64::MAX;
    let mut mips_max: f64 = 0.0;
    let mut mips_avg_count: u128 = 0;

    let timing_scale = 1_000;

    loop {
        //println!("State: {:08x?}", state);

        let tb = tb_mgr.get_tb(&ctx, state.pc, bus)?;

        if state.pc == prev_pc && tb.count_uniq == 2 {
            break;
        }
        prev_pc = state.pc;

        tb.execute(state, bus, &mut tb_mgr)?;
        icount += tb.count_uniq;

        if icount > timing_scale {
            let elapsed_micros_tot = now.elapsed().as_micros();
            let elapsed_micros = elapsed_micros_tot - prev_elapsed;
            prev_elapsed = elapsed_micros_tot;
            let elapsed = (elapsed_micros as f64) / 1_000_000.0;
            let mips = (icount as f64) / elapsed / 1_000_000.0;
            mips_min = f64::min(mips_min, mips);
            mips_max = f64::max(mips_max, mips);
            mips_avg += mips;
            mips_avg_count += 1;

            icount_tot += icount;
            icount = 0;
        }
    }

    let elapsed_micros = now.elapsed().as_micros();
    let elapsed = (elapsed_micros as f64) / 1_000_000.0;

    println!("CpuState: {:x?}", state);
    println!("elapsed time: {}", elapsed);
    println!("icount: {}", icount_tot + icount);
    println!("MIPS (average): {}", mips_avg / (mips_avg_count as f64));
    println!("MIPS (min): {}", mips_min);
    println!("MIPS (max): {}", mips_max);

    Ok(())
}
