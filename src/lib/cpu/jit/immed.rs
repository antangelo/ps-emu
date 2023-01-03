use super::decode;
use super::TranslationBlock;

impl<'ctx> TranslationBlock<'ctx> {
    pub(super) fn emit_addiu(&mut self, instr: &decode::MipsIInstr) {
        if self.finalized || instr.t_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let i32_type = self.ctx.i32_type();
        let immed = (instr.immediate as i16) as i32;
        let const_imm = i32_type.const_int(immed as u64, true);

        let dest_reg =
            self.gep_gp_register(instr.t_reg, &format!("addiu_{}_dest", self.count_uniq));

        let src_reg = self.get_gpr_value(instr.s_reg, &format!("addiu_{}", self.count_uniq));

        let add_res =
            self.builder
                .build_int_add(src_reg, const_imm, &format!("addiu_{}", self.count_uniq));

        self.instr_finished_emitting();
        self.builder.build_store(dest_reg, add_res);
    }

    pub(super) fn emit_andi(&mut self, instr: &decode::MipsIInstr) {
        if self.finalized || instr.t_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let i32_type = self.ctx.i32_type();

        let s_val = self.get_gpr_value(instr.s_reg, &format!("andi_{}", self.count_uniq));
        let immed = i32_type.const_int(instr.immediate as u64, true);

        let t_reg = self.gep_gp_register(instr.t_reg, &format!("andi_{}_t_reg", self.count_uniq));

        let and_val =
            self.builder
                .build_and(s_val, immed, &format!("andi_{}_res", self.count_uniq));

        self.instr_finished_emitting();
        self.builder.build_store(t_reg, and_val);
    }

    pub(super) fn emit_ori(&mut self, instr: &decode::MipsIInstr) {
        if self.finalized || instr.t_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let i32_type = self.ctx.i32_type();

        let s_val = self.get_gpr_value(instr.s_reg, &format!("ori_{}", self.count_uniq));
        let immed = i32_type.const_int(instr.immediate as u64, true);

        let t_reg = self.gep_gp_register(instr.t_reg, &format!("ori_{}_t_reg", self.count_uniq));

        let or_val = self
            .builder
            .build_or(s_val, immed, &format!("ori_{}_res", self.count_uniq));

        self.instr_finished_emitting();
        self.builder.build_store(t_reg, or_val);
    }

    pub(super) fn emit_xori(&mut self, instr: &decode::MipsIInstr) {
        if self.finalized || instr.t_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let i32_type = self.ctx.i32_type();

        let s_val = self.get_gpr_value(instr.s_reg, &format!("xori_{}", self.count_uniq));
        let immed = i32_type.const_int(instr.immediate as u64, true);

        let t_reg = self.gep_gp_register(instr.t_reg, &format!("xori_{}_t_reg", self.count_uniq));

        let xor_val =
            self.builder
                .build_xor(s_val, immed, &format!("xori_{}_res", self.count_uniq));

        self.instr_finished_emitting();
        self.builder.build_store(t_reg, xor_val);
    }

    pub(super) fn emit_slti(&mut self, instr: &decode::MipsIInstr) {
        if instr.t_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let s_val = self.get_gpr_value(instr.s_reg, &format!("slti_{}_s", self.count_uniq));

        let i32_type = self.ctx.i32_type();
        let immed = i32_type.const_int(((instr.immediate as i16) as i32) as u64, true);

        let cmp_val = self.builder.build_int_compare(
            inkwell::IntPredicate::SLT,
            s_val,
            immed,
            &format!("slti_{}_cmp", self.count_uniq),
        );
        let cmp_zext = self.builder.build_int_z_extend(
            cmp_val,
            i32_type,
            &format!("slti_{}_zext", self.count_uniq),
        );
        let t_reg = self.gep_gp_register(instr.t_reg, &format!("slti_{}_t_reg", self.count_uniq));

        self.instr_finished_emitting();
        self.builder.build_store(t_reg, cmp_zext);
    }

    pub(super) fn emit_sltiu(&mut self, instr: &decode::MipsIInstr) {
        if instr.t_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let s_val = self.get_gpr_value(instr.s_reg, &format!("sltiu_{}_s", self.count_uniq));

        let i32_type = self.ctx.i32_type();
        let immed = i32_type.const_int(((instr.immediate as i16) as i32) as u64, true);

        let cmp_val = self.builder.build_int_compare(
            inkwell::IntPredicate::ULT,
            s_val,
            immed,
            &format!("sltiu_{}_cmp", self.count_uniq),
        );
        let cmp_zext = self.builder.build_int_z_extend(
            cmp_val,
            i32_type,
            &format!("sltiu_{}_zext", self.count_uniq),
        );
        let t_reg = self.gep_gp_register(instr.t_reg, &format!("sltiu_{}_t_reg", self.count_uniq));

        self.instr_finished_emitting();
        self.builder.build_store(t_reg, cmp_zext);
    }

    pub(super) fn emit_lui(&mut self, instr: &decode::MipsIInstr) {
        if self.finalized || instr.t_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let i32_type = self.ctx.i32_type();

        let t_reg = self.gep_gp_register(instr.t_reg, &format!("lui_{}_t_reg", self.count_uniq));
        let immed = i32_type.const_int((instr.immediate as u64) << 16, true);

        self.instr_finished_emitting();
        self.builder.build_store(t_reg, immed);
    }
}

#[cfg(test)]
mod test {
    use crate::cpu::jit::harness::TestHarness;

    #[test]
    fn jit_test_addiu() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();
        let val: u32 = 42;

        th.push_dummy_load(1);
        th.push_instr("addiu", 0, 0, 1, val as u16, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], val);
    }

    #[test]
    fn jit_test_andi() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();
        let val: u32 = 42;

        th.push_instr("ori", 0, 0, 1, val as u16, 0);
        th.push_dummy_load(1);
        th.push_instr("andi", 0, 1, 1, 0xf, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], val & 0xf);
    }

    #[test]
    fn jit_test_ori() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();
        let val: u32 = 42;

        th.push_dummy_load(1);
        th.push_instr("ori", 0, 0, 1, val as u16, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], val);
    }

    #[test]
    fn jit_test_xori() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();
        let val: u32 = 42;

        th.push_instr("ori", 0, 0, 1, val as u16, 0);
        th.push_dummy_load(1);
        th.push_instr("xori", 0, 1, 1, val as u16, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 0);
    }

    #[test]
    fn jit_test_sltiu() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.push_instr("ori", 0, 0, 1, 42, 0);
        th.push_dummy_load(2);
        th.push_instr("sltiu", 0, 1, 2, 100, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 42);
        assert_eq!(state.gpr[1], 1);
    }

    #[test]
    fn jit_test_slti() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.push_instr("ori", 0, 0, 1, 42, 0);
        th.push_dummy_load(2);
        th.push_instr("slti", 0, 1, 2, 100, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 42);
        assert_eq!(state.gpr[1], 1);
    }

    #[test]
    fn jit_test_lui() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.push_dummy_load(1);
        th.push_instr("lui", 0, 0, 1, 0x1000, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 0x1000 << 16);
    }
}
