use super::decode;
use super::{DelaySlotArg, TranslationBlock};

impl<'ctx> TranslationBlock<'ctx> {
    fn branch_delay_slot_action<'a, 'b>(tb: &'a mut TranslationBlock<'b>) {
        let arg = tb.delay_slot_arg.as_ref().unwrap();
        let i32_type = tb.ctx.i32_type();

        let immed_i = arg.immed as i16;
        let target = (immed_i as i32) * 4;

        let curr_pc_off = (tb.count_uniq * 4) as i32;
        let target_taken = i32_type.const_int((target + curr_pc_off) as u64, true);

        let target_not_taken = i32_type.const_int((tb.count_uniq * 4 + 4) as u64, false);

        let target_v = tb.builder.build_select(
            arg.value.into_int_value(),
            target_taken,
            target_not_taken,
            &format!("b_select_{}", arg.count),
        );

        let pc_ptr = tb.gep_pc(&format!("beq_{}", arg.count));
        let pc_val = tb
            .builder
            .build_load(pc_ptr, &format!("beq_{}_pc_val", arg.count));

        let next_pc = tb.builder.build_int_add(
            pc_val.into_int_value(),
            target_v.into_int_value(),
            &format!("beq_{}_next_pc", arg.count),
        );

        tb.builder.build_store(pc_ptr, next_pc);
        tb.finalized = true;
    }

    pub(super) fn emit_bgtz(&mut self, instr: &decode::MipsIInstr) {
        let s_val = self.get_gpr_value(instr.s_reg, &format!("bgtz_{}", self.count_uniq));
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

    pub(super) fn emit_blez(&mut self, instr: &decode::MipsIInstr) {
        let s_val = self.get_gpr_value(instr.s_reg, &format!("blez_{}", self.count_uniq));
        let i32_type = self.ctx.i32_type();
        let zero = i32_type.const_zero();
        let cmp = self.builder.build_int_compare(
            inkwell::IntPredicate::SLE,
            s_val,
            zero,
            &format!("blez_{}_cmp", self.count_uniq),
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

    pub(super) fn emit_beq(&mut self, instr: &decode::MipsIInstr) {
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

    pub(super) fn emit_bne(&mut self, instr: &decode::MipsIInstr) {
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
}

#[cfg(test)]
mod test {
    use crate::cpu::jit::harness::TestHarness;

    #[test]
    fn jit_test_beq_taken() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();
        let target = 0x100;

        th.push_instr("beq", 0, 0, 0, target, 0);
        th.push_instr("sll", 0, 0, 0, 0, 0);

        th.execute(&mut state).unwrap();

        assert_eq!(state.pc, 0x1000 + 4 + (target << 2) as u32);
    }

    #[test]
    fn jit_test_beq_not_taken() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();
        let target = 0x100;

        th.push_instr("addiu", 0, 0, 1, 1, 0);
        th.push_instr("beq", 0, 1, 0, target, 0);
        th.push_instr("sll", 0, 0, 0, 0, 0);

        th.execute(&mut state).unwrap();

        assert_eq!(state.pc, 0x1000 + 12);
    }

    #[test]
    fn jit_test_bne_not_taken() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();
        let target = 0x100;

        th.push_instr("bne", 0, 0, 0, target, 0);
        th.push_instr("sll", 0, 0, 0, 0, 0);

        th.execute(&mut state).unwrap();

        assert_eq!(state.pc, 0x1000 + 8);
    }

    #[test]
    fn jit_test_bne_taken() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();
        let target = 0x100;

        th.push_instr("addiu", 0, 0, 1, 1, 0);
        th.push_instr("bne", 0, 1, 0, target, 0);
        th.push_instr("sll", 0, 0, 0, 0, 0);

        th.execute(&mut state).unwrap();

        assert_eq!(state.pc, 0x1000 + 8 + (target << 2) as u32);
    }

    #[test]
    fn jit_test_blez_not_taken() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();
        let target = 0x100;

        th.push_instr("addiu", 0, 0, 1, 1, 0);
        th.push_instr("blez", 0, 1, 0, target, 0);
        th.push_instr("sll", 0, 0, 0, 0, 0);

        th.execute(&mut state).unwrap();

        assert_eq!(state.pc, 0x1000 + 0xc);
    }

    #[test]
    fn jit_test_blez_taken() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();
        let target = 0x100;

        th.push_instr("addiu", 0, 0, 1, -1 as i16 as u16, 0);
        th.push_instr("blez", 0, 1, 0, target, 0);
        th.push_instr("sll", 0, 0, 0, 0, 0);

        th.execute(&mut state).unwrap();

        assert_eq!(state.pc, 0x1000 + 8 + (target << 2) as u32);
    }

    #[test]
    fn jit_test_bgtz_not_taken() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();
        let target = 0x100;

        th.push_instr("bgtz", 0, 0, 0, target, 0);
        th.push_instr("sll", 0, 0, 0, 0, 0);

        th.execute(&mut state).unwrap();

        assert_eq!(state.pc, 0x1000 + 8);
    }

    #[test]
    fn jit_test_bgtz_taken() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();
        let target = 0x100;

        th.push_instr("addiu", 0, 0, 1, 1, 0);
        th.push_instr("bgtz", 0, 1, 0, target, 0);
        th.push_instr("sll", 0, 0, 0, 0, 0);

        th.execute(&mut state).unwrap();

        assert_eq!(state.pc, 0x1000 + 8 + (target << 2) as u32);
    }
}
