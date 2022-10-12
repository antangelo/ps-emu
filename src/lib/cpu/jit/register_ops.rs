use super::decode;
use super::TranslationBlock;

impl<'ctx> TranslationBlock<'ctx> {
    pub(super) fn emit_addu(&mut self, instr: &decode::MipsRInstr) {
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

    pub(super) fn emit_or(&mut self, instr: &decode::MipsRInstr) {
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

    pub(super) fn emit_sll(&mut self, instr: &decode::MipsRInstr) {
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

    pub(super) fn emit_sltu(&mut self, instr: &decode::MipsRInstr) {
        if instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let i32_type = self.ctx.i32_type();
        let s_val = self.get_gpr_value(instr.s_reg, &format!("sltu_{}_s", self.count_uniq));
        let t_val = self.get_gpr_value(instr.t_reg, &format!("sltu_{}_t", self.count_uniq));
        let cmp_val = self.builder.build_int_compare(
            inkwell::IntPredicate::ULT,
            s_val,
            t_val,
            &format!("sltu_{}_cmp", self.count_uniq),
        );
        let cmp_zext = self.builder.build_int_z_extend(
            cmp_val,
            i32_type,
            &format!("sltu_{}_zext", self.count_uniq),
        );

        let d_reg = self.gep_gp_register(instr.d_reg, &format!("sltu_{}_d", self.count_uniq));
        self.builder.build_store(d_reg, cmp_zext);

        self.instr_finished_emitting();
    }
}
