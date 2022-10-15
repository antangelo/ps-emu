use super::decode;
use super::TranslationBlock;

impl<'ctx> TranslationBlock<'ctx> {
    pub(super) fn emit_load_sized(&mut self, size: u32, instr: &decode::MipsIInstr, sext: bool) {
        if self.finalized || instr.t_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let bool_type = self.ctx.bool_type();
        let i8_type = self.ctx.i8_type();
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
        let _read_success = self.mem_read(
            addr.into(),
            size_v.into(),
            i8_type.const_int(instr.t_reg as u64, false).into(),
            bool_type.const_int(sext as u64, false).into(),
            &format!("{}_{}_read", instr.opcode, self.count_uniq),
        );

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

#[cfg(test)]
mod test {
    use crate::cpu::jit::harness::TestHarness;

    #[test]
    fn jit_test_sw_lw() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let addr = 0x1400;
        let val = 42;

        th.push_instr("addiu", 0, 0, 1, addr, 0);
        th.push_instr("addiu", 0, 0, 2, val as u16, 0);
        th.push_instr("sw", 0, 1, 2, 0, 0);
        th.push_instr("lw", 0, 1, 3, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[2], val);
    }

    #[test]
    fn jit_test_sb_lb() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let addr = 0x1400;
        let val = -10;

        th.push_instr("addiu", 0, 0, 1, addr, 0);
        th.push_instr("addiu", 0, 0, 2, val as u16, 0);
        th.push_instr("sb", 0, 1, 2, 0, 0);
        th.push_instr("lb", 0, 1, 3, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[2], val as u32);
    }

    #[test]
    fn jit_test_sb_lbu() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let addr = 0x1400;
        let val: i8 = -10;

        th.push_instr("addiu", 0, 0, 1, addr, 0);
        th.push_instr("addiu", 0, 0, 2, val as u8 as u16, 0);
        th.push_instr("sb", 0, 1, 2, 0, 0);
        th.push_instr("lbu", 0, 1, 3, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[2], val as u8 as u32);
    }
}
