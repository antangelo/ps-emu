use inkwell::values::IntValue;

use super::decode;
use super::TranslationBlock;

impl<'ctx> TranslationBlock<'ctx> {
    fn emit_left_shift(&mut self, instr: &decode::MipsRInstr, shamt: IntValue) {
        if self.finalized || instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let d_reg = self.gep_gp_register(
            instr.d_reg,
            &format!("{}_{}_d_reg", instr.function, self.count_uniq),
        );
        let t_val = self.get_gpr_value(
            instr.t_reg,
            &format!("{}_{}", instr.function, self.count_uniq),
        );

        let sll_val = self.builder.build_left_shift(
            t_val,
            shamt,
            &format!("{}_{}_res", instr.function, self.count_uniq),
        );

        self.builder.build_store(d_reg, sll_val);

        self.instr_finished_emitting();
    }

    pub(super) fn emit_sllv(&mut self, instr: &decode::MipsRInstr) {
        let s_reg = self.get_gpr_value(instr.s_reg, &format!("sllv_s_reg_{}", self.count_uniq));
        self.emit_left_shift(instr, s_reg);
    }

    pub(super) fn emit_sll(&mut self, instr: &decode::MipsRInstr) {
        let i8_type = self.ctx.i8_type();
        let shamt = i8_type.const_int(instr.shamt as u64, false);

        self.emit_left_shift(instr, shamt);
    }

    fn emit_right_shift(&mut self, instr: &decode::MipsRInstr, shamt: IntValue, sign_extend: bool) {
        if self.finalized || instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let d_reg = self.gep_gp_register(
            instr.d_reg,
            &format!("{}_{}_d_reg", instr.function, self.count_uniq),
        );
        let t_val = self.get_gpr_value(
            instr.t_reg,
            &format!("{}_{}", instr.function, self.count_uniq),
        );

        let sll_val = self.builder.build_right_shift(
            t_val,
            shamt,
            sign_extend,
            &format!("{}_{}_res", instr.function, self.count_uniq),
        );

        self.builder.build_store(d_reg, sll_val);

        self.instr_finished_emitting();
    }

    pub(super) fn emit_srl(&mut self, instr: &decode::MipsRInstr) {
        let i8_type = self.ctx.i8_type();
        let shamt = i8_type.const_int(instr.shamt as u64, false);
        self.emit_right_shift(instr, shamt, false);
    }

    pub(super) fn emit_sra(&mut self, instr: &decode::MipsRInstr) {
        let i8_type = self.ctx.i8_type();
        let shamt = i8_type.const_int(instr.shamt as u64, false);
        self.emit_right_shift(instr, shamt, true);
    }

    pub(super) fn emit_slrv(&mut self, instr: &decode::MipsRInstr) {
        let s_reg = self.get_gpr_value(instr.s_reg, &format!("srlv_s_reg_{}", self.count_uniq));
        self.emit_right_shift(instr, s_reg, false);
    }

    pub(super) fn emit_srav(&mut self, instr: &decode::MipsRInstr) {
        let s_reg = self.get_gpr_value(instr.s_reg, &format!("srav_s_reg_{}", self.count_uniq));
        self.emit_right_shift(instr, s_reg, true);
    }

    pub(super) fn emit_add(&mut self, instr: &decode::MipsRInstr) {
        if self.finalized || instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let d_reg = self.gep_gp_register(instr.d_reg, &format!("add_{}_d_reg", self.count_uniq));

        let s_val = self.get_gpr_value(instr.s_reg, &format!("add_{}", self.count_uniq));
        let t_val = self.get_gpr_value(instr.t_reg, &format!("add_{}", self.count_uniq));

        // FIXME: Throw arithmetic exception on signed overflow
        let add_val =
            self.builder
                .build_int_add(s_val, t_val, &format!("add_{}_res", self.count_uniq));

        self.builder.build_store(d_reg, add_val);

        self.instr_finished_emitting();
    }

    pub(super) fn emit_addu(&mut self, instr: &decode::MipsRInstr) {
        if self.finalized || instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let d_reg = self.gep_gp_register(instr.d_reg, &format!("addu_{}_d_reg", self.count_uniq));

        let s_val = self.get_gpr_value(instr.s_reg, &format!("addu_{}", self.count_uniq));
        let t_val = self.get_gpr_value(instr.t_reg, &format!("addu_{}", self.count_uniq));

        let add_val =
            self.builder
                .build_int_add(s_val, t_val, &format!("addu_{}_res", self.count_uniq));

        self.builder.build_store(d_reg, add_val);

        self.instr_finished_emitting();
    }

    pub(super) fn emit_sub(&mut self, instr: &decode::MipsRInstr) {
        if self.finalized || instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let d_reg = self.gep_gp_register(instr.d_reg, &format!("sub_{}_d_reg", self.count_uniq));

        let s_val = self.get_gpr_value(instr.s_reg, &format!("sub_{}", self.count_uniq));
        let t_val = self.get_gpr_value(instr.t_reg, &format!("sub_{}", self.count_uniq));

        // FIXME: Throw integer overflow exception
        let sub_val =
            self.builder
                .build_int_sub(s_val, t_val, &format!("sub_{}_res", self.count_uniq));

        self.builder.build_store(d_reg, sub_val);

        self.instr_finished_emitting();
    }

    pub(super) fn emit_subu(&mut self, instr: &decode::MipsRInstr) {
        if self.finalized || instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let d_reg = self.gep_gp_register(instr.d_reg, &format!("sub_{}_d_reg", self.count_uniq));

        let s_val = self.get_gpr_value(instr.s_reg, &format!("sub_{}", self.count_uniq));
        let t_val = self.get_gpr_value(instr.t_reg, &format!("sub_{}", self.count_uniq));

        let sub_val =
            self.builder
                .build_int_sub(s_val, t_val, &format!("sub_{}_res", self.count_uniq));

        self.builder.build_store(d_reg, sub_val);

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

    pub(super) fn emit_nor(&mut self, instr: &decode::MipsRInstr) {
        if self.finalized || instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let d_reg = self.gep_gp_register(instr.d_reg, &format!("nor_{}_d_reg", self.count_uniq));

        let s_val = self.get_gpr_value(instr.s_reg, &format!("nor_{}", self.count_uniq));
        let t_val = self.get_gpr_value(instr.t_reg, &format!("nor_{}", self.count_uniq));

        let or_val = self
            .builder
            .build_or(s_val, t_val, &format!("nor_{}_res", self.count_uniq));

        let nor_val = self
            .builder
            .build_not(or_val, &format!("nor_{}_nor", self.count_uniq));

        self.builder.build_store(d_reg, nor_val);

        self.instr_finished_emitting();
    }

    pub(super) fn emit_xor(&mut self, instr: &decode::MipsRInstr) {
        if self.finalized || instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let d_reg = self.gep_gp_register(instr.d_reg, &format!("xor_{}_d_reg", self.count_uniq));

        let s_val = self.get_gpr_value(instr.s_reg, &format!("xor_{}", self.count_uniq));
        let t_val = self.get_gpr_value(instr.t_reg, &format!("xor_{}", self.count_uniq));

        let xor_val = self
            .builder
            .build_xor(s_val, t_val, &format!("xor_{}_res", self.count_uniq));

        self.builder.build_store(d_reg, xor_val);

        self.instr_finished_emitting();
    }

    pub(super) fn emit_and(&mut self, instr: &decode::MipsRInstr) {
        if self.finalized || instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let d_reg = self.gep_gp_register(instr.d_reg, &format!("and_{}_d_reg", self.count_uniq));

        let s_val = self.get_gpr_value(instr.s_reg, &format!("and_{}", self.count_uniq));
        let t_val = self.get_gpr_value(instr.t_reg, &format!("and_{}", self.count_uniq));

        let and_val = self
            .builder
            .build_and(s_val, t_val, &format!("and_{}_res", self.count_uniq));

        self.builder.build_store(d_reg, and_val);

        self.instr_finished_emitting();
    }

    fn emit_int_compare(&mut self, instr: &decode::MipsRInstr, pred: inkwell::IntPredicate) {
        if instr.d_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let i32_type = self.ctx.i32_type();
        let s_val = self.get_gpr_value(instr.s_reg, &format!("sltu_{}_s", self.count_uniq));
        let t_val = self.get_gpr_value(instr.t_reg, &format!("sltu_{}_t", self.count_uniq));
        let cmp_val = self.builder.build_int_compare(
            pred,
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

    pub(super) fn emit_sltu(&mut self, instr: &decode::MipsRInstr) {
        self.emit_int_compare(instr, inkwell::IntPredicate::ULT);
    }

    pub(super) fn emit_slt(&mut self, instr: &decode::MipsRInstr) {
        self.emit_int_compare(instr, inkwell::IntPredicate::SLT);
    }
}

#[cfg(test)]
mod test {
    use crate::cpu::jit::harness::TestHarness;

    #[test]
    fn jit_test_addu() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.push_instr("addiu", 0, 0, 1, 40, 0);
        th.push_instr("addiu", 0, 0, 2, 2, 0);
        th.push_instr("addu", 1, 1, 2, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 42);
    }

    #[test]
    fn jit_test_add() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.push_instr("addiu", 0, 0, 1, 40, 0);
        th.push_instr("addiu", 0, 0, 2, 2, 0);
        th.push_instr("add", 1, 1, 2, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 42);
    }

    #[test]
    fn jit_test_subu() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.load32(1, 10);
        th.load32(2, 5);
        th.push_instr("subu", 1, 1, 2, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 5);
    }

    #[test]
    fn jit_test_sub() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.load32(1, 10);
        th.load32(2, 5);
        th.push_instr("sub", 1, 1, 2, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 5);
    }

    #[test]
    fn jit_test_or() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.push_instr("addiu", 0, 0, 1, 1, 0);
        th.push_instr("addiu", 0, 0, 2, 2, 0);
        th.push_instr("or", 1, 1, 2, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 3);
    }

    #[test]
    fn jit_test_nor() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.load32(1, 0x0000ffff);
        th.load32(2, 0xff000000);
        th.push_instr("nor", 1, 1, 2, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 0x00ff0000);
    }

    #[test]
    fn jit_test_xor() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.load32(1, 0x8);
        th.load32(2, 0x9);
        th.push_instr("xor", 1, 1, 2, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 1);
    }

    #[test]
    fn jit_test_and() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.load32(1, 0x8);
        th.load32(2, 0x9);
        th.push_instr("and", 1, 1, 2, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 0x8);
    }

    #[test]
    fn jit_test_sll() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.push_instr("addiu", 0, 0, 1, 1, 0);
        th.push_instr("sll", 1, 0, 1, 1, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 2);
    }

    #[test]
    fn jit_test_srl() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.load32(1, 4);
        th.push_instr("srl", 1, 0, 1, 1, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 2);
    }

    #[test]
    fn jit_test_sra() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.load32(1, -2 as i32 as u32);
        th.push_instr("sra", 1, 0, 1, 1, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], -1 as i32 as u32);
    }

    #[test]
    fn jit_test_sllv() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.load32(1, 1);
        th.load32(2, 2);
        th.push_instr("sllv", 1, 2, 1, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 4);
    }

    #[test]
    fn jit_test_slrv() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.load32(1, 4);
        th.load32(2, 2);
        th.push_instr("slrv", 1, 2, 1, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 1);
    }

    #[test]
    fn jit_test_srav() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.load32(1, -2 as i32 as u32);
        th.load32(2, 1);
        th.push_instr("srav", 1, 2, 1, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], -1 as i32 as u32);
    }

    #[test]
    fn jit_test_sltu_negative() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.push_instr("addiu", 0, 0, 1, 1, 0);
        th.push_instr("sltu", 1, 1, 0, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 0);
    }

    #[test]
    fn jit_test_sltu_positive() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.push_instr("addiu", 0, 0, 1, 1, 0);
        th.push_instr("sltu", 1, 0, 1, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 1);
    }

    #[test]
    fn jit_test_slt_negative() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.push_instr("addiu", 0, 0, 1, 1, 0);
        th.push_instr("slt", 1, 1, 0, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 0);
    }

    #[test]
    fn jit_test_slt_positive() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.push_instr("addiu", 0, 0, 1, 1, 0);
        th.push_instr("slt", 1, 0, 1, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 1);
    }
}
