use super::decode;
use super::TranslationBlock;

impl<'ctx> TranslationBlock<'ctx> {
    pub(super) fn emit_mflo(&mut self, instr: &decode::MipsRInstr) {
        if self.finalized || instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let d_reg = self.gep_gp_register(instr.d_reg, &format!("mflo_{}_d_reg", self.count_uniq));
        let lo_ptr = self.gep_lo(&format!("mflo_{}_lo_ptr", self.count_uniq));
        let lo = self
            .builder
            .build_load(lo_ptr, &format!("mflo_{}_lo", self.count_uniq));
        self.builder.build_store(d_reg, lo);

        self.instr_finished_emitting();
    }

    pub(super) fn emit_mfhi(&mut self, instr: &decode::MipsRInstr) {
        if self.finalized || instr.d_reg == 0 {
            self.instr_finished_emitting();
        }

        let d_reg = self.gep_gp_register(instr.d_reg, &format!("mfhi_{}_d_reg", self.count_uniq));
        let hi_ptr = self.gep_hi(&format!("mfhi_{}_hi_ptr", self.count_uniq));
        let hi = self
            .builder
            .build_load(hi_ptr, &format!("mfhi_{}_hi", self.count_uniq));
        self.builder.build_store(d_reg, hi);

        self.instr_finished_emitting();
    }

    pub(super) fn emit_divu(&mut self, instr: &decode::MipsRInstr) {
        let s_reg = self.get_gpr_value(instr.s_reg, &format!("divu_{}_s", self.count_uniq));
        let t_reg = self.get_gpr_value(instr.t_reg, &format!("divu_{}_t", self.count_uniq));

        let div = self.builder.build_int_unsigned_div(
            s_reg,
            t_reg,
            &format!("divu_{}_quotient", self.count_uniq),
        );
        let lo = self.gep_lo(&format!("divu_{}_lo", self.count_uniq));
        self.builder.build_store(lo, div);

        let modulo = self.builder.build_int_unsigned_rem(
            s_reg,
            t_reg,
            &format!("divu_{}_mod", self.count_uniq),
        );
        let hi = self.gep_hi(&format!("divu_{}_hi", self.count_uniq));
        self.builder.build_store(hi, modulo);

        self.instr_finished_emitting();
    }

    pub(super) fn emit_mult(&mut self, instr: &decode::MipsRInstr) {
        let s_reg = self.get_gpr_value(instr.s_reg, &format!("mult_{}_s", self.count_uniq));
        let t_reg = self.get_gpr_value(instr.t_reg, &format!("mult_{}_t", self.count_uniq));

        let i64_type = self.ctx.i64_type();
        let s_ext = self.builder.build_int_s_extend(
            s_reg,
            i64_type,
            &format!("mult_{}_s_ext", self.count_uniq),
        );
        let t_ext = self.builder.build_int_s_extend(
            t_reg,
            i64_type,
            &format!("mult_{}_t_ext", self.count_uniq),
        );

        let mult_v =
            self.builder
                .build_int_mul(s_ext, t_ext, &format!("mult_{}_v", self.count_uniq));

        let i32_type = self.ctx.i32_type();
        let mult_hi = self.builder.build_right_shift(
            mult_v,
            i64_type.const_int(32, false),
            false,
            &format!("mult_{}_hi", self.count_uniq),
        );
        let hi_reg = self.gep_hi(&format!("mult_{}_hi_reg", self.count_uniq));
        let mult_hi_32 = self.builder.build_int_truncate(
            mult_hi,
            i32_type,
            &format!("mult_{}_hi_cast", self.count_uniq),
        );
        self.builder.build_store(hi_reg, mult_hi_32);

        let mult_lo = self.builder.build_int_truncate(
            mult_v,
            i32_type,
            &format!("mult_{}_lo", self.count_uniq),
        );
        let lo_reg = self.gep_lo(&format!("mult_{}_lo_reg", self.count_uniq));
        self.builder.build_store(lo_reg, mult_lo);

        self.instr_finished_emitting();
    }
}
