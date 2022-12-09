use super::bus::{BusDevice, SizedReadResult};
use super::jit::CpuState;
use super::{decode, opcode};
use std::rc::Rc;

mod itype;
mod jtype;
mod mem;
mod rtype;

type BusType = super::bus_vec::VecBus;
type TbDynFunc<'ctx> =
    unsafe extern "C" fn(state: *mut CpuState, bus: *mut BusType, mgr: *mut TbManager<'ctx>);

fn new_tb<'ctx>(
    id: u64,
    ctx: &'ctx inkwell::context::Context,
) -> Result<ThreadBlock<'ctx>, String> {
    let module = ctx.create_module(&format!("tb_mod_{}", id));
    let ee = module
        .create_jit_execution_engine(inkwell::OptimizationLevel::Less)
        .map_err(|e| e.to_string())?;
    let builder = ctx.create_builder();

    let i32_type = ctx.i32_type();
    let mips_state_type = ctx.opaque_struct_type("mips_state");
    mips_state_type.set_body(&[i32_type.into(); 34], false);

    let bus_type = ctx
        .opaque_struct_type("mips_bus")
        .ptr_type(inkwell::AddressSpace::Generic);
    let state_type = module
        .get_struct_type("mips_state")
        .unwrap()
        .ptr_type(inkwell::AddressSpace::Generic);
    let tb_mgr_type = ctx
        .opaque_struct_type("tb_manager")
        .ptr_type(inkwell::AddressSpace::Generic);

    let void_type = ctx.void_type();
    let fn_type = void_type.fn_type(
        &[state_type.into(), bus_type.into(), tb_mgr_type.into()],
        false,
    );
    let func_name = format!("tb_func_{}", id);
    let func = module.add_function(&func_name, fn_type, None);

    let block = ctx.append_basic_block(func, &func_name);
    builder.position_at_end(block);

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

    let i8_type = ctx.i8_type();
    let i16_type = ctx.i16_type();
    let r_fn_type = void_type.fn_type(
        &[
            i8_type.into(),
            i8_type.into(),
            i8_type.into(),
            i8_type.into(),
            state_type.into(),
            bus_type.into(),
            tb_mgr_type.into(),
        ],
        false,
    );

    let r_jmp_fn_type = i32_type.fn_type(
        &[
            i8_type.into(),
            i8_type.into(),
            i8_type.into(),
            i8_type.into(),
            state_type.into(),
            bus_type.into(),
            tb_mgr_type.into(),
            i32_type.into(),
        ],
        false,
    );

    let i_fn_type = void_type.fn_type(
        &[
            i8_type.into(),
            i8_type.into(),
            i16_type.into(),
            state_type.into(),
            bus_type.into(),
            tb_mgr_type.into(),
        ],
        false,
    );

    let i_jmp_fn_type = i32_type.fn_type(
        &[
            i8_type.into(),
            i8_type.into(),
            i16_type.into(),
            state_type.into(),
            bus_type.into(),
            tb_mgr_type.into(),
            i32_type.into(),
        ],
        false,
    );

    let j_fn_type = i32_type.fn_type(
        &[
            i32_type.into(),
            state_type.into(),
            bus_type.into(),
            tb_mgr_type.into(),
            i32_type.into(),
        ],
        false,
    );

    let mut tb = ThreadBlock {
        id,
        icount: 0,
        finalized: false,
        ctx,
        module,
        ee,
        builder,
        func,
        state_arg,
        bus_arg,
        mgr_arg,
        tb_func: None,
        delay_slot_arg: None,
        delay_slot_fn: None,
    };

    tb.register_rtypes(&r_jmp_fn_type, &r_fn_type);
    tb.register_itypes(&i_jmp_fn_type, &i_fn_type);
    tb.register_jtypes(&j_fn_type);

    Ok(tb)
}

struct DelaySlotArg<'ctx> {
    count: u64,
    value: inkwell::values::BasicValueEnum<'ctx>,
}

pub(super) struct ThreadBlock<'ctx> {
    id: u64,
    icount: u64,
    finalized: bool,

    ctx: &'ctx inkwell::context::Context,
    module: inkwell::module::Module<'ctx>,
    ee: inkwell::execution_engine::ExecutionEngine<'ctx>,
    builder: inkwell::builder::Builder<'ctx>,

    func: inkwell::values::FunctionValue<'ctx>,

    state_arg: inkwell::values::PointerValue<'ctx>,
    bus_arg: inkwell::values::PointerValue<'ctx>,
    mgr_arg: inkwell::values::PointerValue<'ctx>,

    tb_func: Option<inkwell::execution_engine::JitFunction<'ctx, TbDynFunc<'ctx>>>,
    delay_slot_arg: Option<DelaySlotArg<'ctx>>,
    delay_slot_fn: Option<fn(&mut ThreadBlock)>,
}

impl<'ctx> ThreadBlock<'ctx> {
    fn emit_nop(&mut self) {
        self.emit_rtype(&decode::MipsRInstr {
            s_reg: 0,
            t_reg: 0,
            d_reg: 0,
            shamt: 0,
            function: opcode::MipsFunction::Sll,
        });
    }

    fn gep_pc(&self, prefix: &str) -> inkwell::values::PointerValue<'ctx> {
        self.builder
            .build_struct_gep(self.state_arg, 33, &format!("{}_pc", prefix))
            .unwrap()
    }

    fn instr_finished_emitting(&mut self) {
        if self.finalized {
            return;
        }

        if let Some(f) = self.delay_slot_fn {
            f(self);
            self.delay_slot_fn = None;
            self.delay_slot_arg = None;
        }

        self.icount += 1;
    }

    fn translate(&mut self, bus: &mut dyn BusDevice, pc: u32) -> Result<(), String> {
        let mut addr = pc;
        while !self.finalized {
            let read_result = bus.read(addr, 32).map_err(|_| "Failed to read instr")?;
            if let SizedReadResult::Dword(instr_raw) = read_result {
                let instr = decode::mips_decode(instr_raw);

                match instr {
                    decode::MipsInstr::RType(r) => self.emit_rtype(&r),
                    decode::MipsInstr::IType(i) => self.emit_itype(&i),
                    decode::MipsInstr::JType(j) => self.emit_jtype(&j),
                    _ => self.emit_nop(),
                }
            } else {
                panic!(
                    "Read of size 32 didn't return dword, instead have {:?}",
                    read_result
                );
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
        }

        assert!(self.delay_slot_fn.is_none());
        Ok(())
    }

    pub fn finalize(&mut self) {
        unsafe { self.tb_func = self.ee.get_function(&format!("tb_func_{}", self.id)).ok() }
    }

    pub(crate) fn execute(
        &self,
        state: &mut CpuState,
        bus: &mut BusType,
        mgr: &mut TbManager<'ctx>,
    ) -> Result<(), String> {
        if let Some(func) = self.tb_func.as_ref() {
            unsafe { func.call(state, bus, mgr) }
            Ok(())
        } else {
            Err(String::from("Failed to compile TB"))
        }
    }

    fn register_rtype_fn(
        &mut self,
        fn_type: &inkwell::types::FunctionType<'ctx>,
        mips_func: opcode::MipsFunction,
        func: usize,
    ) {
        let name = format!("rtype_fn_{}", mips_func);
        let mod_fn = self.module.add_function(&name, *fn_type, None);
        self.ee.add_global_mapping(&mod_fn, func as usize);
    }

    fn register_rtype_jmp_fn(
        &mut self,
        fn_type: &inkwell::types::FunctionType<'ctx>,
        mips_func: opcode::MipsFunction,
        func: usize,
    ) {
        let name = format!("rtype_jmp_fn_{}", mips_func);
        let mod_fn = self.module.add_function(&name, *fn_type, None);
        self.ee.add_global_mapping(&mod_fn, func);
    }

    fn register_itype_fn(
        &mut self,
        fn_type: &inkwell::types::FunctionType<'ctx>,
        opcode: opcode::MipsOpcode,
        func: usize,
    ) {
        let name = format!("itype_fn_{}", opcode);
        let mod_fn = self.module.add_function(&name, *fn_type, None);
        self.ee.add_global_mapping(&mod_fn, func);
    }

    fn register_itype_jmp_fn(
        &mut self,
        fn_type: &inkwell::types::FunctionType<'ctx>,
        opcode: opcode::MipsOpcode,
        func: usize,
    ) {
        let name = format!("itype_jmp_fn_{}", opcode);
        let mod_fn = self.module.add_function(&name, *fn_type, None);
        self.ee.add_global_mapping(&mod_fn, func);
    }

    fn register_jtype_fn(
        &mut self,
        fn_type: &inkwell::types::FunctionType<'ctx>,
        opcode: opcode::MipsOpcode,
        func: usize,
    ) {
        let name = format!("jtype_fn_{}", opcode);
        let mod_fn = self.module.add_function(&name, *fn_type, None);
        self.ee.add_global_mapping(&mod_fn, func as usize);
    }
}

pub(crate) struct TbManager<'ctx> {
    trie: super::trie::Trie<ThreadBlock<'ctx>>,
}

impl<'ctx> TbManager<'ctx> {
    pub(super) fn new() -> Self {
        Self {
            trie: super::trie::Trie::default(),
        }
    }

    pub(super) fn get_tb(
        &mut self,
        ctx: &'ctx inkwell::context::Context,
        addr: u32,
        bus: &mut impl BusDevice,
    ) -> Result<Rc<ThreadBlock<'ctx>>, String> {
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
        let tb = tb_mgr.get_tb(&ctx, state.pc, bus)?;

        if state.pc == prev_pc && tb.icount == 2 {
            break;
        }

        prev_pc = state.pc;

        tb.execute(state, bus, &mut tb_mgr)?;
        icount += tb.icount;

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
