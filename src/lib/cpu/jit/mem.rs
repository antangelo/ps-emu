use super::decode;
use super::TranslationBlock;

impl<'ctx> TranslationBlock<'ctx> {
    pub(super) fn emit_load_sized(&mut self, size: u32, instr: &decode::MipsIInstr, sext: bool) {
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

    pub(super) fn emit_lb(&mut self, instr: &decode::MipsIInstr) {
        self.emit_load_sized(8, instr, true);
    }

    pub(super) fn emit_lbu(&mut self, instr: &decode::MipsIInstr) {
        self.emit_load_sized(8, instr, false);
    }

    pub(super) fn emit_lw(&mut self, instr: &decode::MipsIInstr) {
        self.emit_load_sized(32, instr, false);
    }

    pub(super) fn emit_sb(&mut self, instr: &decode::MipsIInstr) {
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

    pub(super) fn emit_sw(&mut self, instr: &decode::MipsIInstr) {
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
}
