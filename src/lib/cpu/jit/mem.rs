use super::decode;
use super::TranslationBlock;

impl<'ctx> TranslationBlock<'ctx> {
    fn load_delay_slot_action<'a, 'b>(tb: &'a mut TranslationBlock<'b>) {
        tb.apply_load_delay_if_present();
        tb.delay_slot_load_register = None;
    }

    fn decode_vaddr(&self, instr: &decode::MipsIInstr) -> inkwell::values::IntValue<'ctx> {
        let i32_type = self.ctx.i32_type();
        let offset = i32_type.const_int(instr.immediate as u64, true);

        let base_val = self.get_gpr_value(
            instr.s_reg,
            &format!("{}_{}", instr.opcode, self.count_uniq),
        );

        self.builder.build_int_add(
            base_val,
            offset,
            &format!("{}_{}_addr", instr.opcode, self.count_uniq),
        )
    }

    pub(super) fn emit_load_sized(&mut self, size: u32, instr: &decode::MipsIInstr, sext: bool) {
        // FIXME: Support t_reg = 0
        assert_ne!(instr.t_reg, 0);

        if self.finalized {
            self.instr_finished_emitting();
            return;
        }

        let bool_type = self.ctx.bool_type();
        let i8_type = self.ctx.i8_type();
        let i32_type = self.ctx.i32_type();

        let addr = self.decode_vaddr(instr);

        let size_v = i32_type.const_int(size as u64, false);

        // Finish emitting first, so that successive loads don't overwrite each other's save data
        let count = self.count_uniq;
        self.instr_finished_emitting();

        let _read_success = self.mem_read(
            addr.into(),
            size_v.into(),
            i8_type.const_int(instr.t_reg as u64, false).into(),
            bool_type.const_int(sext as u64, false).into(),
            &format!("{}_{}_read", instr.opcode, count),
        );

        // FIXME: Check result of mem read

        // If the block is finished, then the load will complete at the beginning of the next block
        if self.finalized {
            return;
        }

        self.delay_slot_load_register = Some(instr.t_reg);
        self.delay_slot_hazard = Some(Self::load_delay_slot_action);
    }

    pub(super) fn emit_lb(&mut self, instr: &decode::MipsIInstr) {
        self.emit_load_sized(8, instr, true);
    }

    pub(super) fn emit_lbu(&mut self, instr: &decode::MipsIInstr) {
        self.emit_load_sized(8, instr, false);
    }

    pub(super) fn emit_lh(&mut self, instr: &decode::MipsIInstr) {
        self.emit_load_sized(16, instr, true);
    }

    pub(super) fn emit_lhu(&mut self, instr: &decode::MipsIInstr) {
        self.emit_load_sized(16, instr, false);
    }

    pub(super) fn emit_lw(&mut self, instr: &decode::MipsIInstr) {
        self.emit_load_sized(32, instr, false);
    }

    fn emit_store_sized(&mut self, size: u32, instr: &decode::MipsIInstr) {
        let i32_type = self.ctx.i32_type();

        let addr = self.decode_vaddr(instr);
        let t_val = self.get_gpr_value(
            instr.t_reg,
            &format!("{}_{}", instr.opcode, self.count_uniq),
        );

        let size = i32_type.const_int(size as u64, false);
        let _mem_res = self.mem_write(
            addr.into(),
            size.into(),
            t_val.into(),
            &format!("sb_{}_write", self.count_uniq),
        );

        // FIXME: Check result of mem write

        self.instr_finished_emitting();
    }

    pub(super) fn emit_sb(&mut self, instr: &decode::MipsIInstr) {
        self.emit_store_sized(8, instr);
    }

    pub(super) fn emit_sh(&mut self, instr: &decode::MipsIInstr) {
        self.emit_store_sized(16, instr);
    }

    pub(super) fn emit_sw(&mut self, instr: &decode::MipsIInstr) {
        self.emit_store_sized(32, instr);
    }

    fn emit_unaligned_load(&mut self, instr: &decode::MipsIInstr, left: bool) {
        // FIXME: Support t_reg = 0
        assert_ne!(instr.t_reg, 0);

        if self.finalized {
            self.instr_finished_emitting();
            return;
        }

        let i32_type = self.ctx.i32_type();
        let i8_type = self.ctx.i8_type();
        let bool_type = self.ctx.bool_type();

        let addr = self.decode_vaddr(instr);
        let delay_val_ptr =
            self.gep_load_delay_value(&format!("{}_{}_delay_gep", instr.opcode, self.count_uniq));

        // Test if the previous instruction was a load to the same register
        // If so, we can reuse that load's delay value without waiting
        // If not, then we have to grab the source register value directly
        let source_val = self
            .delay_slot_load_register
            .filter(|reg| *reg == instr.t_reg)
            .map_or_else(
                || self.get_gpr_value(instr.t_reg, &format!("{}_{}_gpr_t", instr.opcode, self.count_uniq)),
                |_| {
                    self.builder
                        .build_load(
                            delay_val_ptr,
                            &format!("{}_{}_delay_read", instr.opcode, self.count_uniq),
                        )
                        .into_int_value()
                },
            );

        let addr_aligned = self.builder.build_and(
            addr,
            i32_type.const_int(0xffff_fffc, false),
            &format!("{}_{}_align", instr.opcode, self.count_uniq),
        );

        // mem_read will overwrite the load delay register, so apply any staggered loads that are
        // waiting, provided the target register is not the same as this one.
        // If the target is the same, then the load is not applied until after this instruction, at
        // least.
        let count = self.count_uniq;
        if self
            .delay_slot_load_register
            .filter(|reg| *reg == instr.t_reg)
            .is_some()
        {
            self.delay_slot_hazard = None;
        }

        self.instr_finished_emitting();

        // FIXME: Check memory access success
        let _read_success = self.mem_read(
            addr_aligned.into(),
            i32_type.const_int(32, false).into(),
            i8_type.const_int(instr.t_reg as u64, false).into(),
            bool_type.const_zero().into(),
            &format!("{}_{}_read", instr.opcode, count),
        );

        let mem_read_val = self
            .builder
            .build_load(delay_val_ptr, &format!("{}_{}_mem_read", instr.opcode, self.count_uniq));

        let alignment_bytes = self.builder.build_and(
            addr,
            i32_type.const_int(0x0000_0003, false),
            &format!("{}_{}_alignment_bytes", instr.opcode, self.count_uniq),
        );
        let alignment = self.builder.build_int_mul(
            alignment_bytes,
            i32_type.const_int(8, false),
            &format!("{}_{}_alignment", instr.opcode, self.count_uniq),
        );

        let inv_alignment = self.builder.build_int_sub(
            i32_type.const_int(24, false),
            alignment,
            &format!("{}_{}_mem_read_shift_bytes", instr.opcode, self.count_uniq),
        );

        let mem_shift: inkwell::values::IntValue;
        let curr_val_mask: inkwell::values::IntValue;
        if left {
            mem_shift = self.builder.build_left_shift(
                mem_read_val.into_int_value(),
                inv_alignment,
                &format!("{}_{}_mem_shift", instr.opcode, self.count_uniq),
                );

            curr_val_mask = self.builder.build_right_shift(
                i32_type.const_int(0x00ff_ffff, false),
                alignment,
                false,
                &format!("{}_{}_curr_mask", instr.opcode, self.count_uniq),
                );
        } else {
            mem_shift = self.builder.build_right_shift(
                mem_read_val.into_int_value(),
                alignment,
                false,
                &format!("{}_{}_mem_shift", instr.opcode, self.count_uniq),
                );
            
            curr_val_mask = self.builder.build_left_shift(
                i32_type.const_int(0xffff_ff00, false),
                inv_alignment,
                &format!("{}_{}_curr_mask", instr.opcode, self.count_uniq),
                );
        }

        let curr_val = self.builder.build_and(
            source_val,
            curr_val_mask,
            &format!("{}_{}_source_masked", instr.opcode, self.count_uniq),
        );

        let new_val = self.builder.build_or(
            curr_val,
            mem_shift,
            &format!("{}_{}_final_value", instr.opcode, self.count_uniq),
        );

        // Update the delay value to the adjusted read value
        // The register is already set correctly by mem_read
        self.builder.build_store(delay_val_ptr, new_val);

        if self.finalized {
            return;
        }

        self.delay_slot_load_register = Some(instr.t_reg);
        self.delay_slot_hazard = Some(Self::load_delay_slot_action);
    }

    pub(super) fn emit_lwl(&mut self, instr: &decode::MipsIInstr) {
        self.emit_unaligned_load(instr, true);
    }

    pub(super) fn emit_lwr(&mut self, instr: &decode::MipsIInstr) {
        self.emit_unaligned_load(instr, false);
    }
}

#[cfg(test)]
mod test {
    use crate::cpu::jit::harness::TestHarness;

    // TODO: Test load into zero register performs RAM access
    // TODO: Test overwriting load target register in delay slot has correct priority

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

        println!("{:08x?}", state);
        assert_eq!(state.gpr[2], val);
    }

    #[test]
    fn jit_test_load_delay_slot() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let addr = 0x1400;
        let val = 42;
        let delay_imm = 10;

        th.push_instr("addiu", 0, 0, 1, addr, 0);
        th.push_instr("addiu", 0, 0, 2, val as u16, 0);
        th.load32(3, delay_imm);
        th.push_instr("sw", 0, 1, 2, 0, 0);
        th.push_instr("lw", 0, 1, 3, 0, 0);
        th.push_instr("addu", 4, 3, 0, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[2], val);
        assert_eq!(state.gpr[3], delay_imm);
    }

    #[test]
    fn jit_test_load_in_branch_delay_slot() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let addr = 0x1400;
        let val = 42;
        let delay_imm = 10;

        th.push_instr("addiu", 0, 0, 1, addr, 0);
        th.push_instr("addiu", 0, 0, 2, val as u16, 0);
        th.load32(3, delay_imm);
        th.push_instr("sw", 0, 1, 2, 0, 0);

        // Manually insert branch, no need to call th.finish()
        th.push_instr("bne", 0, 0, 0, 0, 0);
        th.push_instr("lw", 0, 1, 3, 0, 0);

        // Second execution point
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[2], delay_imm);

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
    fn jit_test_sh_lh() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let addr = 0x1400;
        let val = -10;

        th.push_instr("addiu", 0, 0, 1, addr, 0);
        th.push_instr("addiu", 0, 0, 2, val as u16, 0);
        th.push_instr("sh", 0, 1, 2, 0, 0);
        th.push_instr("lh", 0, 1, 3, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[2], val as u32);
    }

    #[test]
    fn jit_test_sh_lhu() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let addr = 0x1400;
        let val = -10;

        th.push_instr("addiu", 0, 0, 1, addr, 0);
        th.push_instr("addiu", 0, 0, 2, val as u16, 0);
        th.push_instr("sh", 0, 1, 2, 0, 0);
        th.push_instr("lhu", 0, 1, 3, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[2], val as u16 as u32);
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

    #[test]
    fn jit_test_lwl() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let addr = 0x1400;
        let delay_val = 10;

        th.load32(10, 0xffffdead);
        th.load32(11, 0xbeefffff);

        th.push_instr("addiu", 0, 0, 1, addr, 0);
        th.push_instr("sw", 0, 1, 10, 0, 0);
        th.push_instr("sw", 0, 1, 11, 4, 0);
        th.push_instr("addiu", 0, 0, 3, delay_val, 0);
        th.push_instr("lwl", 0, 1, 3, 1, 0);
        th.push_instr("addu", 4, 3, 0, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[3], delay_val as u32);
        assert_eq!(state.gpr[2], 0xdead_0000 + delay_val as u32);
    }

    #[test]
    fn jit_test_lwr() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let addr = 0x1400;
        let delay_val = 10;

        th.load32(10, 0xffffdead);
        th.load32(11, 0xbeefffff);

        th.push_instr("addiu", 0, 0, 1, addr, 0);
        th.push_instr("sw", 0, 1, 10, 0, 0);
        th.push_instr("sw", 0, 1, 11, 4, 0);
        th.push_instr("addiu", 0, 0, 3, delay_val, 0);
        th.push_instr("lwr", 0, 1, 3, 6, 0);
        th.push_instr("addu", 4, 3, 0, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[3], delay_val as u32);
        assert_eq!(state.gpr[2], 0x0000_beef);
    }

    #[test]
    fn jit_test_lwr_lwl() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::jit::CpuState::default();

        let addr = 0x1400;
        let delay_val = 10;

        th.load32(10, 0xffffdead);
        th.load32(11, 0xbeefffff);

        th.push_instr("addiu", 0, 0, 1, addr, 0);
        th.push_instr("sw", 0, 1, 10, 0, 0);
        th.push_instr("sw", 0, 1, 11, 4, 0);
        th.push_instr("addiu", 0, 0, 3, delay_val, 0);
        th.push_instr("lwl", 0, 1, 3, 1, 0);
        th.push_instr("lwr", 0, 1, 3, 6, 0);
        th.push_instr("addu", 4, 3, 0, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        assert_eq!(state.gpr[3], delay_val as u32);
        assert_eq!(state.gpr[2], 0xdeadbeef);
    }
}
