use super::{decode, opcode};
use inkwell::values::AnyValue;

type TbDynFunc =
    unsafe extern "C" fn(state: *mut CpuState, bus: *mut super::bus::Bus, scratch: *mut u32);

pub fn new_tb<'ctx>(
    id: u64,
    ctx: &'ctx inkwell::context::Context,
) -> Result<TranslationBlock<'ctx>, String> {
    let module = ctx.create_module(&format!("tb_mod_{}", id));
    let ee = module
        .create_jit_execution_engine(inkwell::OptimizationLevel::Aggressive)
        .map_err(|e| e.to_string())?;
    let builder = ctx.create_builder();

    let i32_type = ctx.i32_type();
    let state_type = ctx.opaque_struct_type("mips_state");
    state_type.set_body(&[i32_type.into(); 34], false);

    let bus_type = ctx
        .opaque_struct_type("mips_bus")
        .ptr_type(inkwell::AddressSpace::Generic);
    let i32_ptr_type = i32_type.ptr_type(inkwell::AddressSpace::Generic);
    let bool_type = ctx.bool_type();

    let read_fn = module.add_function(
        "tb_mem_read",
        i32_type.fn_type(
            &[
                bus_type.into(),
                i32_type.into(),
                i32_type.into(),
                i32_ptr_type.into(),
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
    let state_type = module
        .get_struct_type("mips_state")
        .unwrap()
        .ptr_type(inkwell::AddressSpace::Generic);
    let fn_type = void_type.fn_type(
        &[state_type.into(), bus_type.into(), i32_ptr_type.into()],
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
    let scratch_arg = func
        .get_nth_param(2)
        .ok_or("No scratcharg")?
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
        func_block: block,
        state_arg,
        bus_arg,
        scratch_arg,
        delay_slot_hazard: None,
        delay_slot_arg: None,
        tb_func: None,
    })
}

struct TbManager<'ctx> {
    tb_cache: std::collections::HashMap<u32, TranslationBlock<'ctx>>,
}

impl<'ctx> TbManager<'ctx> {
    fn new() -> Self {
        Self {
            tb_cache: std::collections::HashMap::default(),
        }
    }

    fn get_tb(
        &mut self,
        ctx: &'ctx inkwell::context::Context,
        addr: u32,
        bus: &mut super::bus::Bus,
    ) -> Result<&TranslationBlock<'ctx>, String> {
        if !self.tb_cache.contains_key(&addr) {
            let mut tb = new_tb(addr as u64, ctx)?;
            tb.translate(bus, addr)?;
            tb.finalize();
            self.tb_cache.insert(addr, tb);
        }

        let tb = self.tb_cache.get(&addr);
        Ok(tb.unwrap())
    }
}

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
    func_block: inkwell::basic_block::BasicBlock<'ctx>,

    state_arg: inkwell::values::PointerValue<'ctx>,
    bus_arg: inkwell::values::PointerValue<'ctx>,
    scratch_arg: inkwell::values::PointerValue<'ctx>,

    delay_slot_hazard: Option<fn(&mut TranslationBlock)>,
    delay_slot_arg: Option<DelaySlotArg<'ctx>>,

    tb_func: Option<inkwell::execution_engine::JitFunction<'ctx, TbDynFunc>>,
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
            .build_struct_gep(self.state_arg, reg as u32 - 1, name)
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

    fn mem_read(
        &self,
        addr: inkwell::values::BasicMetadataValueEnum<'ctx>,
        size: inkwell::values::BasicMetadataValueEnum<'ctx>,
        dest: inkwell::values::BasicMetadataValueEnum<'ctx>,
        name: &str,
    ) -> inkwell::values::BasicValueEnum<'ctx> {
        let read_fn = self.module.get_function("tb_mem_read").unwrap();
        self.builder
            .build_call(read_fn, &[self.bus_arg.into(), addr, size, dest], name)
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
            .build_call(write_fn, &[self.bus_arg.into(), addr, size, value], name)
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

    fn emit_addiu(&mut self, instr: &decode::MipsIInstr) {
        if self.finalized || instr.t_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let i32_type = self.ctx.i16_type();
        let const_imm = i32_type.const_int(instr.immediate.into(), true);

        let dest_reg =
            self.gep_gp_register(instr.t_reg, &format!("addiu_{}_dest", self.count_uniq));

        let src_reg = self.get_gpr_value(instr.s_reg, &format!("addiu_{}", self.count_uniq));

        let add_res =
            self.builder
                .build_int_add(src_reg, const_imm, &format!("addiu_{}", self.count_uniq));
        self.builder.build_store(dest_reg, add_res);

        self.instr_finished_emitting();
    }

    fn emit_addu(&mut self, instr: &decode::MipsRInstr) {
        if self.finalized || instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let d_reg = self.gep_gp_register(instr.d_reg, &format!("addu_{}_d_reg", self.count_uniq));

        let s_val = self.get_gpr_value(instr.s_reg, &format!("addu_{}", self.count_uniq));
        let t_val = self.get_gpr_value(instr.t_reg, &format!("addu_{}", self.count_uniq));

        let or_val =
            self.builder
                .build_int_add(s_val, t_val, &format!("addu_{}_res", self.count_uniq));

        self.builder.build_store(d_reg, or_val);

        self.instr_finished_emitting();
    }

    fn branch_delay_slot_action<'a, 'b>(tb: &'a mut TranslationBlock<'b>) {
        let arg = tb.delay_slot_arg.as_ref().unwrap();
        let i32_type = tb.ctx.i32_type();

        let taken = tb
            .ctx
            .insert_basic_block_after(tb.func_block, &format!("beq_{}_taken", arg.count));

        {
            tb.builder.position_at_end(taken);

            // FIXME: This is stupid
            let immed_i = arg.immed as i16;
            let target = (immed_i as i32) * 4;

            let curr_pc_off = (tb.count_uniq * 4) as i32;
            let target_v = i32_type.const_int((target + curr_pc_off) as u64, true);

            let pc_ptr = tb.gep_pc(&format!("beq_{}", arg.count));
            let pc_val = tb
                .builder
                .build_load(pc_ptr, &format!("beq_{}_pc_val", arg.count));

            let next_pc = tb.builder.build_int_add(
                pc_val.into_int_value(),
                target_v,
                &format!("beq_{}_next_pc", arg.count),
                );
            tb.builder.build_store(pc_ptr, next_pc);
            tb.builder.build_return(None);
        }

        let not_taken = tb
            .ctx
            .insert_basic_block_after(taken, &format!("beq_{}_not_taken", arg.count));

        {
            tb.builder.position_at_end(not_taken);
            let target_v = i32_type.const_int((tb.count_uniq * 4 + 4) as u64, false);

            let pc_ptr = tb.gep_pc(&format!("beq_{}", arg.count));
            let pc_val = tb
                .builder
                .build_load(pc_ptr, &format!("beq_{}_pc_val", arg.count));

            let next_pc = tb.builder.build_int_add(
                pc_val.into_int_value(),
                target_v,
                &format!("beq_{}_next_pc", arg.count),
                );
            tb.builder.build_store(pc_ptr, next_pc);
            tb.builder.build_return(None);
        }

        tb.builder.position_at_end(tb.func_block);
        tb.builder
            .build_conditional_branch(arg.value.into_int_value(), taken, not_taken);
        tb.finalized = true;
    }

    fn emit_bgtz(&mut self, instr: &decode::MipsIInstr) {
        let s_val = self.get_gpr_value(instr.s_reg, &format!("beq_{}", self.count_uniq));
        let i32_type = self.ctx.i32_type();
        let zero = i32_type.const_zero();
        let cmp = self.builder.build_int_compare(
            inkwell::IntPredicate::SGT,
            s_val,
            zero,
            &format!("bgtz_{}_cmp", self.count_uniq),
        );

        let count = self.count_uniq;
        self.instr_finished_emitting();

        self.delay_slot_arg = Some(DelaySlotArg {
            count,
            immed: instr.immediate,
            value: cmp.into(),
        });
        self.delay_slot_hazard = Some(Self::branch_delay_slot_action);
    }

    fn emit_beq(&mut self, instr: &decode::MipsIInstr) {
        let s_val = self.get_gpr_value(instr.s_reg, &format!("beq_{}", self.count_uniq));
        let t_val = self.get_gpr_value(instr.t_reg, &format!("beq_{}", self.count_uniq));
        let cmp = self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            s_val,
            t_val,
            &format!("beq_{}_cmp", self.count_uniq),
        );

        let count = self.count_uniq;
        self.instr_finished_emitting();

        self.delay_slot_arg = Some(DelaySlotArg {
            count,
            immed: instr.immediate,
            value: cmp.into(),
        });
        self.delay_slot_hazard = Some(Self::branch_delay_slot_action);
    }

    fn emit_bne(&mut self, instr: &decode::MipsIInstr) {
        let s_val = self.get_gpr_value(instr.s_reg, &format!("bne_{}", self.count_uniq));
        let t_val = self.get_gpr_value(instr.t_reg, &format!("bne_{}", self.count_uniq));
        let cmp = self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            s_val,
            t_val,
            &format!("bne_{}_cmp", self.count_uniq),
        );

        let count = self.count_uniq;
        self.instr_finished_emitting();

        self.delay_slot_arg = Some(DelaySlotArg {
            count,
            immed: instr.immediate,
            value: cmp.into(),
        });
        self.delay_slot_hazard = Some(Self::branch_delay_slot_action);
    }

    fn emit_load_sized(&mut self, size: u32, instr: &decode::MipsIInstr, sext: bool) {
        if self.finalized || instr.t_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let i32_type = self.ctx.i32_type();

        let offset = i32_type.const_int(instr.immediate as u64, true);

        let base_val = self.get_gpr_value(
            instr.s_reg,
            &format!("{}_{}", instr.opcode, self.count_uniq),
        );
        let addr = self.builder.build_int_add(
            base_val,
            offset,
            &format!("{}_{}_addr", instr.opcode, self.count_uniq),
        );

        let size_v = i32_type.const_int(size as u64, false);
        let read_val = self.mem_read(
            addr.into(),
            size_v.into(),
            self.scratch_arg.into(),
            &format!("{}_{}_read", instr.opcode, self.count_uniq),
        );

        // Using dt in the load delay slot is UB, so we don't need to emulate anything fancy
        let t_reg = self.gep_gp_register(
            instr.t_reg,
            &format!("{}_{}_t_reg", instr.opcode, self.count_uniq),
        );

        if sext {
            let sext_val = self.builder.build_int_s_extend(
                read_val.into_int_value(),
                i32_type,
                &format!("{}_{}_sext", instr.opcode, self.count_uniq),
            );
            self.builder.build_store(t_reg, sext_val);
        } else {
            self.builder.build_store(t_reg, read_val);
        }

        // FIXME: Check result of mem read

        self.instr_finished_emitting();
    }

    fn emit_lb(&mut self, instr: &decode::MipsIInstr) {
        self.emit_load_sized(8, instr, true);
    }

    fn emit_lbu(&mut self, instr: &decode::MipsIInstr) {
        self.emit_load_sized(8, instr, false);
    }

    fn emit_lui(&mut self, instr: &decode::MipsIInstr) {
        if self.finalized || instr.t_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let i32_type = self.ctx.i32_type();

        let t_reg = self.gep_gp_register(instr.t_reg, &format!("lui_{}_t_reg", self.count_uniq));
        let immed = i32_type.const_int((instr.immediate as u64) << 16, true);

        self.builder.build_store(t_reg, immed);

        self.instr_finished_emitting();
    }

    fn emit_lw(&mut self, instr: &decode::MipsIInstr) {
        self.emit_load_sized(32, instr, false);
    }

    fn emit_j(&mut self, instr: &decode::MipsJInstr) {
        if self.finalized {
            return;
        }

        let i32_type = self.ctx.i32_type();
        let target_v = i32_type.const_int((instr.target << 2) as u64, false);

        let pc = self.gep_pc(&format!("j_{}", self.count_uniq));

        self.builder.build_store(pc, target_v);

        self.instr_finished_emitting();

        self.delay_slot_hazard = Some(|tb| {
            tb.builder.build_return(None);
            tb.finalized = true;
        });
    }

    fn emit_jal(&mut self, instr: &decode::MipsJInstr) {
        let i32_type = self.ctx.i32_type();
        let target_v = i32_type.const_int((instr.target << 2) as u64, false);

        let pc = self.gep_pc(&format!("jal_{}", self.count_uniq));
        let ra = self.gep_gp_register(31, &format!("jal_{}_ra", self.count_uniq));

        let pc_incr = i32_type.const_int(self.count_uniq * 4 + 8, false);

        let pc_val = self
            .builder
            .build_load(pc, &format!("jal_{}_pc_val", self.count_uniq));
        let ra_val = self.builder.build_int_add(
            pc_val.into_int_value(),
            pc_incr,
            &format!("jal_{}_ra_val", self.count_uniq),
        );

        self.builder.build_store(ra, ra_val);
        self.builder.build_store(pc, target_v);

        self.instr_finished_emitting();

        self.delay_slot_hazard = Some(|tb| {
            tb.builder.build_return(None);
            tb.finalized = true;
        });
    }

    fn emit_jr(&mut self, instr: &decode::MipsRInstr) {
        let target_v = self.get_gpr_value(instr.s_reg, &format!("jr_{}", self.count_uniq));

        let pc = self.gep_pc(&format!("jr_{}", self.count_uniq));

        // FIXME: Handle address exception if lower bits of ra are nonzero

        self.builder.build_store(pc, target_v);

        self.instr_finished_emitting();

        self.delay_slot_hazard = Some(|tb| {
            tb.builder.build_return(None);
            tb.finalized = true;
        });
    }

    fn emit_jalr(&mut self, instr: &decode::MipsRInstr) {
        let i32_type = self.ctx.i32_type();
        let target_v = self.get_gpr_value(instr.s_reg, &format!("jalr_{}", self.count_uniq));

        let pc = self.gep_pc(&format!("jal_{}", self.count_uniq));
        let ra = self.gep_gp_register(instr.d_reg, &format!("jalr_{}_d_ra", self.count_uniq));

        let pc_incr = i32_type.const_int(self.count_uniq * 4 + 8, false);

        let pc_val = self
            .builder
            .build_load(pc, &format!("jal_{}_pc_val", self.count_uniq));
        let ra_val = self.builder.build_int_add(
            pc_val.into_int_value(),
            pc_incr,
            &format!("jal_{}_ra_val", self.count_uniq),
        );

        // FIXME: Handle address exception if lower bits of ra are nonzero

        self.builder.build_store(ra, ra_val);
        self.builder.build_store(pc, target_v);

        self.instr_finished_emitting();

        self.delay_slot_hazard = Some(|tb| {
            tb.builder.build_return(None);
            tb.finalized = true;
        });
    }

    fn emit_or(&mut self, instr: &decode::MipsRInstr) {
        if self.finalized || instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let d_reg = self.gep_gp_register(instr.d_reg, &format!("or_{}_d_reg", self.count_uniq));

        let s_val = self.get_gpr_value(instr.s_reg, &format!("or_{}", self.count_uniq));
        let t_val = self.get_gpr_value(instr.t_reg, &format!("or_{}", self.count_uniq));

        let or_val = self
            .builder
            .build_or(s_val, t_val, &format!("or_{}_res", self.count_uniq));

        self.builder.build_store(d_reg, or_val);

        self.instr_finished_emitting();
    }

    fn emit_ori(&mut self, instr: &decode::MipsIInstr) {
        if self.finalized || instr.s_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let i32_type = self.ctx.i32_type();

        let s_reg = self.gep_gp_register(instr.s_reg, &format!("ori_{}_s_reg", self.count_uniq));
        let immed = i32_type.const_int(instr.immediate as u64, true);

        let t_val = self.get_gpr_value(instr.t_reg, &format!("ori_{}", self.count_uniq));

        let or_val = self
            .builder
            .build_or(t_val, immed, &format!("ori_{}_res", self.count_uniq));

        self.builder.build_store(s_reg, or_val);

        self.instr_finished_emitting();
    }

    fn emit_sb(&mut self, instr: &decode::MipsIInstr) {
        let i32_type = self.ctx.i32_type();

        let offset = i32_type.const_int(instr.immediate as u64, true);

        let base_val = self.get_gpr_value(instr.s_reg, &format!("sb_{}", self.count_uniq));
        let addr =
            self.builder
                .build_int_add(base_val, offset, &format!("sb_{}_addr", self.count_uniq));

        let t_val = self.get_gpr_value(instr.t_reg, &format!("sb_{}", self.count_uniq));

        let size = i32_type.const_int(8, false);
        let _mem_res = self.mem_write(
            addr.into(),
            size.into(),
            t_val.into(),
            &format!("sb_{}_write", self.count_uniq),
        );

        // FIXME: Check result of mem write

        self.instr_finished_emitting();
    }

    fn emit_sw(&mut self, instr: &decode::MipsIInstr) {
        let i32_type = self.ctx.i32_type();

        let offset = i32_type.const_int(instr.immediate as u64, true);

        let base_val = self.get_gpr_value(instr.s_reg, &format!("sw_{}", self.count_uniq));
        let addr =
            self.builder
                .build_int_add(base_val, offset, &format!("sw_{}_addr", self.count_uniq));

        let t_val = self.get_gpr_value(instr.t_reg, &format!("sw_{}", self.count_uniq));

        let size = i32_type.const_int(32, false);
        let _mem_res = self.mem_write(
            addr.into(),
            size.into(),
            t_val.into(),
            &format!("sw_{}_write", self.count_uniq),
        );

        // FIXME: Check result of mem write

        self.instr_finished_emitting();
    }

    fn emit_sll(&mut self, instr: &decode::MipsRInstr) {
        if self.finalized || instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let d_reg = self.gep_gp_register(instr.d_reg, &format!("sll_{}_d_reg", self.count_uniq));
        let t_val = self.get_gpr_value(instr.t_reg, &format!("sll_{}", self.count_uniq));

        let i8_type = self.ctx.i8_type();
        let shamt = i8_type.const_int(instr.shamt as u64, false);

        let sll_val =
            self.builder
                .build_left_shift(t_val, shamt, &format!("sll_{}_res", self.count_uniq));

        self.builder.build_store(d_reg, sll_val);

        self.instr_finished_emitting();
    }

    fn emit_mflo(&mut self, instr: &decode::MipsRInstr) {
        let d_reg = self.gep_gp_register(instr.d_reg, &format!("mflo_{}_d_reg", self.count_uniq));
        let lo_ptr = self.gep_lo(&format!("mflo_{}_lo_ptr", self.count_uniq));
        let lo = self.builder.build_load(lo_ptr, &format!("mflo_{}_lo", self.count_uniq));
        self.builder.build_store(d_reg, lo);

        self.instr_finished_emitting();
    }

    fn emit_mfhi(&mut self, instr: &decode::MipsRInstr) {
        let d_reg = self.gep_gp_register(instr.d_reg, &format!("mfhi_{}_d_reg", self.count_uniq));
        let hi_ptr = self.gep_lo(&format!("mfhi_{}_hi_ptr", self.count_uniq));
        let hi = self.builder.build_load(hi_ptr, &format!("mfhi_{}_hi", self.count_uniq));
        self.builder.build_store(d_reg, hi);

        self.instr_finished_emitting();
    }

    fn emit_divu(&mut self, instr: &decode::MipsRInstr) {
        let s_reg = self.get_gpr_value(instr.s_reg, &format!("divu_{}_s", self.count_uniq));
        let t_reg = self.get_gpr_value(instr.t_reg, &format!("divu_{}_t", self.count_uniq));

        let div = self.builder.build_int_unsigned_div(s_reg, t_reg, &format!("divu_{}_quotient", self.count_uniq));
        let lo = self.gep_lo(&format!("divu_{}_lo", self.count_uniq));
        self.builder.build_store(lo, div);

        let modulo = self.builder.build_int_unsigned_rem(s_reg, t_reg, &format!("divu_{}_mod", self.count_uniq));
        let hi = self.gep_hi(&format!("divu_{}_hi", self.count_uniq));
        self.builder.build_store(hi, modulo);

        self.instr_finished_emitting();
    }

    fn emit_r_instr(&mut self, instr: &decode::MipsRInstr) {
        match instr.function {
            opcode::MipsFunction::Sll => self.emit_sll(instr),
            opcode::MipsFunction::Jr => self.emit_jr(instr),
            opcode::MipsFunction::Jalr => self.emit_jalr(instr),
            opcode::MipsFunction::AddU => self.emit_addu(instr),
            opcode::MipsFunction::Or => self.emit_or(instr),
            opcode::MipsFunction::Mflo => self.emit_mflo(instr),
            opcode::MipsFunction::Mfhi => self.emit_mfhi(instr),
            opcode::MipsFunction::DivU => self.emit_divu(instr),
            _ => panic!("Not implemented: {}", instr.function),
        }
    }

    fn emit_i_instr(&mut self, instr: &decode::MipsIInstr) {
        match instr.opcode {
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

    pub fn translate(&mut self, bus: &mut super::bus::Bus, pc: u32) -> Result<u32, String> {
        let mut addr = pc;
        while !self.finalized {
            let instr_raw = bus.read(addr, 32).map_err(|_| "Failed to read instr")?;
            let instr = super::decode::mips_decode(instr_raw);

            match instr {
                decode::MipsInstr::RType(r) => self.emit_r_instr(&r),
                decode::MipsInstr::IType(i) => self.emit_i_instr(&i),
                decode::MipsInstr::JType(j) => self.emit_j_instr(&j),
                _ => {
                    self.emit_r_instr(&decode::MipsRInstr{
                        s_reg: 0,
                        t_reg: 0,
                        d_reg: 0,
                        shamt: 0,
                        function: opcode::MipsFunction::Sll
                    });
                    //return Err(format!("Invalid instruction {:#08x}: {:#08x} {}", addr, instr_raw, instr));
                }
            }

            addr += 4;
        }

        Ok(addr)
    }

    pub fn finalize(&mut self) {
        // FIXME: Cache TBs

        unsafe { self.tb_func = self.ee.get_function(&format!("tb_func_{}", self.id)).ok() }
    }
}

impl<'ctx> std::fmt::Display for TranslationBlock<'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.func.print_to_string().fmt(f)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CpuState {
    gpr: [u32; 31],
    hi: u32,
    lo: u32,
    pc: u32,
}

impl CpuState {
    pub fn set_pc(&mut self, pc: u32) {
        self.pc = pc;
    }
}

impl Default for CpuState {
    fn default() -> Self {
        CpuState {
            gpr: [0; 31],
            hi: 0,
            lo: 0,
            pc: 0,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn tb_mem_read(
    bus: *mut super::bus::Bus,
    addr: u32,
    size: u32,
    err: *mut u32,
) -> u32 {
    assert_ne!(std::ptr::null_mut(), bus);
    assert_ne!(std::ptr::null_mut(), err);
    match (*bus).read(addr, size) {
        Ok(v) => {
            *err = 0;
            v
        }
        Err(_) => {
            *err = 1;
            0
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn tb_mem_write(
    bus: *mut super::bus::Bus,
    addr: u32,
    size: u32,
    value: u32,
) -> bool {
    assert_ne!(std::ptr::null_mut(), bus);
    let wv = (*bus).write(addr, size, value);
    match wv {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn execute(bus: &mut super::bus::Bus, state: &mut CpuState) -> Result<(), String> {
    let ctx = inkwell::context::Context::create();
    let mut tb_mgr = TbManager::new();
    let mut scratch: u32 = 0;
    let mut prev_pc = 0;
    let mut icount = 0;
    let now = std::time::Instant::now();

    loop {
        if state.pc == prev_pc {
            break;
        }

        prev_pc = state.pc;

        let tb = tb_mgr.get_tb(&ctx, state.pc, bus)?;
        icount += tb.count_uniq;

        if let Some(func) = tb.tb_func.as_ref() {
            unsafe {
                func.call(
                    core::ptr::addr_of_mut!(*state),
                    core::ptr::addr_of_mut!(*bus),
                    core::ptr::addr_of_mut!(scratch),
                );
            }
        } else {
            panic!("Failed to compile TB");
        }
    }

    let elapsed_micros = now.elapsed().as_micros();
    let elapsed = (elapsed_micros as f64) / 1_000_000.0;

    println!("CpuState: {:x?}", state);
    println!("pc: {:#08x}", state.pc);
    println!("elapsed time: {}", elapsed);
    println!("icount: {}", icount);
    println!("MIPS: {}", (icount as f64) / elapsed / 1_000_000.0);

    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    #[should_panic]
    fn test_tb_write_null() {
        unsafe {
            super::tb_mem_write(core::ptr::null_mut(), 0, 0, 0);
        }
    }

    #[test]
    #[should_panic]
    fn test_tb_read_null_bus() {
        unsafe {
            super::tb_mem_read(core::ptr::null_mut(), 0, 0, core::ptr::null_mut());
        }
    }
}
