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
            return;
        }

        let d_reg = self.gep_gp_register(instr.d_reg, &format!("mfhi_{}_d_reg", self.count_uniq));
        let hi_ptr = self.gep_hi(&format!("mfhi_{}_hi_ptr", self.count_uniq));
        let hi = self
            .builder
            .build_load(hi_ptr, &format!("mfhi_{}_hi", self.count_uniq));
        self.builder.build_store(d_reg, hi);

        self.instr_finished_emitting();
    }

    pub(super) fn emit_mtlo(&mut self, instr: &decode::MipsRInstr) {
        if self.finalized {
            self.instr_finished_emitting();
            return;
        }

        let lo_val = self.get_gpr_value(instr.s_reg, &format!("mtlo_s_val_{}", self.count_uniq));
        let lo_ptr = self.gep_lo(&format!("mtlo_{}_lo_ptr", self.count_uniq));
        self.builder.build_store(lo_ptr, lo_val);

        self.instr_finished_emitting();
    }

    pub(super) fn emit_mthi(&mut self, instr: &decode::MipsRInstr) {
        if self.finalized {
            self.instr_finished_emitting();
            return;
        }

        let hi_val = self.get_gpr_value(instr.s_reg, &format!("mthi_s_val_{}", self.count_uniq));
        let hi_ptr = self.gep_hi(&format!("mthi_{}_lo_ptr", self.count_uniq));
        self.builder.build_store(hi_ptr, hi_val);

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

    pub(super) fn emit_div(&mut self, instr: &decode::MipsRInstr) {
        let s_reg = self.get_gpr_value(instr.s_reg, &format!("div_{}_s", self.count_uniq));
        let t_reg = self.get_gpr_value(instr.t_reg, &format!("div_{}_t", self.count_uniq));

        let div = self.builder.build_int_signed_div(
            s_reg,
            t_reg,
            &format!("divu_{}_quotient", self.count_uniq),
        );
        let lo = self.gep_lo(&format!("div_{}_lo", self.count_uniq));
        self.builder.build_store(lo, div);

        let modulo = self.builder.build_int_signed_rem(
            s_reg,
            t_reg,
            &format!("div_{}_mod", self.count_uniq),
        );
        let hi = self.gep_hi(&format!("div_{}_hi", self.count_uniq));
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

    pub(super) fn emit_multu(&mut self, instr: &decode::MipsRInstr) {
        let s_reg = self.get_gpr_value(instr.s_reg, &format!("multu_{}_s", self.count_uniq));
        let t_reg = self.get_gpr_value(instr.t_reg, &format!("multu_{}_t", self.count_uniq));

        let i64_type = self.ctx.i64_type();
        let s_ext = self.builder.build_int_z_extend(
            s_reg,
            i64_type,
            &format!("multu_{}_s_ext", self.count_uniq),
        );
        let t_ext = self.builder.build_int_z_extend(
            t_reg,
            i64_type,
            &format!("multu_{}_t_ext", self.count_uniq),
        );

        let mult_v =
            self.builder
                .build_int_mul(s_ext, t_ext, &format!("multu_{}_v", self.count_uniq));

        let i32_type = self.ctx.i32_type();
        let mult_hi = self.builder.build_right_shift(
            mult_v,
            i64_type.const_int(32, false),
            false,
            &format!("multu_{}_hi", self.count_uniq),
        );
        let hi_reg = self.gep_hi(&format!("multu_{}_hi_reg", self.count_uniq));
        let mult_hi_32 = self.builder.build_int_truncate(
            mult_hi,
            i32_type,
            &format!("multu_{}_hi_cast", self.count_uniq),
        );
        self.builder.build_store(hi_reg, mult_hi_32);

        let mult_lo = self.builder.build_int_truncate(
            mult_v,
            i32_type,
            &format!("multu_{}_lo", self.count_uniq),
        );
        let lo_reg = self.gep_lo(&format!("multu_{}_lo_reg", self.count_uniq));
        self.builder.build_store(lo_reg, mult_lo);

        self.instr_finished_emitting();
    }
}

#[cfg(test)]
mod test {
    use crate::cpu::jit::harness::TestHarness;

    #[test]
    fn jit_test_mult_with_overflow() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let multiplicand = 0x8000_0000;
        let multiplier = 2;

        th.load32(1, multiplicand);
        th.load32(2, multiplier);
        th.push_instr("mult", 0, 1, 2, 0, 0);
        th.push_instr("mfhi", 1, 0, 0, 0, 0);
        th.push_instr("mflo", 2, 0, 0, 0, 0);

        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.hi, state.gpr[0]);
        assert_eq!(state.lo, state.gpr[1]);

        assert_eq!(state.hi, 0xffffffff);
        assert_eq!(state.lo, 0x0);
    }

    #[test]
    fn jit_test_divu() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let numerator = 5;
        let denominator = 2;

        th.load32(1, numerator);
        th.load32(2, denominator);
        th.push_instr("divu", 0, 1, 2, 0, 0);
        th.push_instr("mfhi", 1, 0, 0, 0, 0);
        th.push_instr("mflo", 2, 0, 0, 0, 0);

        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.hi, state.gpr[0]);
        assert_eq!(state.lo, state.gpr[1]);

        assert_eq!(state.hi, 1);
        assert_eq!(state.lo, 2);
    }

    #[test]
    fn jit_test_mthi_mtlo() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let hi_val = 5;
        let lo_val = 2;

        th.load32(1, hi_val);
        th.load32(2, lo_val);
        th.push_instr("mthi", 0, 1, 0, 0, 0);
        th.push_instr("mtlo", 0, 2, 0, 0, 0);

        th.finish();

        th.execute(&mut state).unwrap();

        println!("{:?}", state);

        assert_eq!(state.hi, state.gpr[0]);
        assert_eq!(state.lo, state.gpr[1]);

        assert_eq!(state.hi, hi_val);
        assert_eq!(state.lo, lo_val);
    }
}
