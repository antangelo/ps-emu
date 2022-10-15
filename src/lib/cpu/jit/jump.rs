use super::decode;
use super::TranslationBlock;

impl<'ctx> TranslationBlock<'ctx> {
    pub(super) fn emit_j(&mut self, instr: &decode::MipsJInstr) {
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

    pub(super) fn emit_jal(&mut self, instr: &decode::MipsJInstr) {
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

    pub(super) fn emit_jr(&mut self, instr: &decode::MipsRInstr) {
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

    pub(super) fn emit_jalr(&mut self, instr: &decode::MipsRInstr) {
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
}

#[cfg(test)]
mod test {
    use crate::cpu::jit::harness::TestHarness;

    #[test]
    fn jit_test_jr_branch_delay_slot_instr_no_edge_case() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.push_instr("jr", 31, 31, 31, 0, 0);
        th.push_instr("addiu", 0, 0, 1, 42, 0);

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[0], 42);
    }

    #[test]
    fn jit_test_j_address_set() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let target = 0x0100;

        th.push_instr("j", 0, 0, 0, 0, target);
        th.push_instr("sll", 0, 0, 0, 0, 0);

        th.execute(&mut state).unwrap();

        assert_eq!(state.pc, target << 2);
    }

    #[test]
    #[ignore = "not implemented"]
    fn jit_test_j_upper_bits_retained() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();
        let target = 0x0100;

        // FIXME: Allow harness to execute in KSEG0,1,2
        th.push_instr("j", 0, 0, 0, 0, target);
        th.push_instr("sll", 0, 0, 0, 0, 0);

        th.execute(&mut state).unwrap();

        assert_eq!(state.pc, target << 2);
    }

    #[test]
    fn jit_test_jal_address_set_ra_set() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let target = 0x0100;

        th.push_instr("jal", 0, 0, 0, 0, target);
        th.push_instr("sll", 0, 0, 0, 0, 0);

        th.execute(&mut state).unwrap();

        assert_eq!(state.pc, target << 2);
        assert_eq!(state.gpr[30], 0x1008);
    }

    #[test]
    fn jit_test_jalr_address_set_ra_set() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let target = 0x0100;

        th.push_instr("ori", 0, 0, 1, target as u16, 0);
        th.push_instr("jalr", 31, 1, 0, 0, 0);
        th.push_instr("sll", 0, 0, 0, 0, 0);

        th.execute(&mut state).unwrap();

        assert_eq!(state.pc, target);
        assert_eq!(state.gpr[30], 0x100c);
    }

    #[test]
    fn jit_test_jr_address_set() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let target = 0x0100;

        th.push_instr("ori", 0, 0, 1, target as u16, 0);
        th.push_instr("jr", 0, 1, 0, 0, 0);
        th.push_instr("sll", 0, 0, 0, 0, 0);

        th.execute(&mut state).unwrap();

        assert_eq!(state.pc, target);
    }

    #[test]
    #[ignore = "not implemented"]
    fn jit_test_jump_in_branch_delay_slot() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        th.push_instr("j", 0, 0, 0, 0, 0x1010); // 0x1000
        th.push_instr("j", 0, 0, 0, 0, 0x2000); // 0x1004
        th.push_instr("addiu", 0, 0, 1, 1, 0); // 0x1008
        th.push_instr("addiu", 0, 0, 2, 2, 0); // 0x100c
        th.push_instr("addiu", 0, 0, 3, 3, 0); // 0x1010
        th.push_instr("addiu", 0, 0, 4, 4, 0); // 0x1014

        th.execute(&mut state).unwrap();

        assert_eq!(state.pc, 0x2000 << 2);
        assert_eq!(state.gpr[0], 1);
        assert_eq!(state.gpr[1], 0);
        assert_eq!(state.gpr[2], 3);
        assert_eq!(state.gpr[3], 0);
    }
}
