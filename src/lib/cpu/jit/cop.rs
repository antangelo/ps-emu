use crate::cpu::{decode::{MipsCopInstr, MipsRInstr}, opcode::MipsCopOperation, cop0};

use super::TranslationBlock;

impl<'ctx> TranslationBlock<'ctx> {
    pub(super) fn emit_cop_operation(&mut self, instr: &MipsCopInstr) {
        match instr.cop {
            0 => self.emit_cop0_operation(instr),
            _ => panic!("Unimplemented COP: {}", instr.cop),
        }
    }

    fn emit_cop0_operation(&mut self, instr: &MipsCopInstr) {
        match instr.operation {
            MipsCopOperation::MoveTo => self.emit_cop0_mtc(instr),
            MipsCopOperation::MoveFrom => self.emit_cop0_mfc(instr),
            _ => panic!("Unimplemented operation {} for CP0", instr.operation),
        }
    }

    fn gep_cop0_reg(&self, reg: u8, name: &str) -> inkwell::values::PointerValue<'ctx> {
        assert!(reg <= 15);
        self.builder.build_struct_gep(self.state_arg, (36 + reg) as u32, name).unwrap()
    }

    pub(super) fn raise_exception(&mut self, cause: &cop0::ExceptionCause, instr: &str, pc_reg: &inkwell::values::PointerValue<'ctx>, curr_pc_val: &inkwell::values::IntValue<'ctx>, main_branch: bool, count: u64) {
        let i32_type = self.ctx.i32_type();

        let pc_incr = if self.finalized {
            // In delay slot, EPC should point to the branch instruction
            // (i.e. the one preceeding this one)
            assert!(count >= 1);
            count * 4 - 4
        } else {
            // Outside of the delay slot, point to the current instruction
            count * 4
        };

        let pc_val = self.builder.build_int_add(
            *curr_pc_val,
            i32_type.const_int(pc_incr as u64, false),
            &format!("{}_{}_pc_val", instr, count),
        );

        let cop0_epc = self.gep_cop0_reg(cop0::Register::Epc as u8, &format!("{}_{}_epc", instr, count));
        self.builder.build_store(cop0_epc, pc_val);

        let mut cop0_cause_val = ((cause.to_int()) << 2) as u32;
        
        if self.finalized {
            cop0_cause_val |= 1 << 31;
        }

        if let cop0::ExceptionCause::CopUnusable(cop) = *cause {
            cop0_cause_val |= ((cop & 0b11) as u32) << 28;
        }

        let cop0_cause_reg = self.gep_cop0_reg(cop0::Register::Cause as u8, &format!("syscall_{}_cause", count));
        self.builder.build_store(cop0_cause_reg, i32_type.const_int(cop0_cause_val as u64, false));

        let cop0_sr = self.gep_cop0_reg(cop0::Register::Sr as u8, &format!("{}_{}_sr", "", count));
        let cop0_sr_val = self.builder.build_load(cop0_sr, &format!("")).into_int_value();

        let cop0_sr_bev = self.builder.build_and(cop0_sr_val, i32_type.const_int(1 << 22, false), &format!(""));
        let cop0_sr_bev_bool = self.builder.build_int_compare(inkwell::IntPredicate::NE, cop0_sr_bev, i32_type.const_zero(), &format!(""));

        let new_pc = self.builder.build_select(cop0_sr_bev_bool, i32_type.const_int(0xbfc0_0180, false), i32_type.const_int(0x8000_0080, false),
        &format!("{}_{}_new_pc", instr, count));
        self.builder.build_store(*pc_reg, new_pc);

        let cop0_sr_mode = self.builder.build_and(cop0_sr_val, i32_type.const_int(0x3f, false), &format!(""));
        let cop0_sr_clear_old_mode = self.builder.build_and(cop0_sr_val, i32_type.const_int(!0x3f, false), &format!(""));
        let cop0_sr_mode_shift = self.builder.build_left_shift(cop0_sr_mode, i32_type.const_int(2, false), &format!(""));
        let cop0_sr_new_mode_masked = self.builder.build_and(cop0_sr_mode_shift, i32_type.const_int(0x3f, false), &format!(""));
        let cop0_sr_new_val = self.builder.build_or(cop0_sr_clear_old_mode, cop0_sr_new_mode_masked, &format!(""));
        self.builder.build_store(cop0_sr, cop0_sr_new_val);

        // No delay slot on exception raise
        if main_branch {
            self.finalized = true;
        } else {
            self.builder.build_return(None);
        }
    }

    pub(super) fn emit_syscall(&mut self, instr: &MipsRInstr) {
        if self.finalized {
            self.instr_finished_emitting();
            return;
        }

        let pc_reg = self.gep_pc(&format!("syscall_{}", self.count_uniq));
        let pc_val = self.builder.build_load(pc_reg, &format!("syscall_{}_pc_val", self.count_uniq)).into_int_value();
        let count = self.count_uniq;

        self.instr_finished_emitting();

        self.raise_exception(&cop0::ExceptionCause::Syscall, &format!("{}", instr.function), &pc_reg, &pc_val, true, count);
    }

    pub(super) fn emit_break(&mut self, instr: &MipsRInstr) {
        if self.finalized {
            self.instr_finished_emitting();
            return;
        }

        let pc_reg = self.gep_pc(&format!("break_{}", self.count_uniq));
        let pc_val = self.builder.build_load(pc_reg, &format!("break_{}_pc_val", self.count_uniq)).into_int_value();
        let count = self.count_uniq;

        self.instr_finished_emitting();

        self.raise_exception(&cop0::ExceptionCause::Break, &format!("{}", instr.function), &pc_reg, &pc_val, true, count);
    }

    fn emit_cop0_mtc(&mut self, instr: &MipsCopInstr) {
        if self.finalized {
            self.instr_finished_emitting();
            return;
        }

        let data = self.get_gpr_value(instr.t_reg, &format!("mtc0_{}_val", self.count_uniq));
        self.instr_finished_emitting();

        let cop_reg = self.gep_cop0_reg(instr.d_reg, &format!("mtc0_{}_cop_reg", self.count_uniq));
        self.builder.build_store(cop_reg, data);
    }

    fn emit_cop0_mfc(&mut self, instr: &MipsCopInstr) {
        if self.finalized || instr.t_reg == 0 {
            self.instr_finished_emitting();
            return;
        }

        let cop_reg = self.gep_cop0_reg(instr.d_reg, &format!("mfc0_{}_cop_reg", self.count_uniq));
        let cop_val = self.builder.build_load(cop_reg, &format!("mfc0_{}_cop_load", self.count_uniq));

        self.instr_finished_emitting();

        let delay_reg = self.gep_load_delay_register(&format!("mfc0_{}_delay_reg", self.count_uniq));
        let i32_type = self.ctx.i32_type();
        self.builder.build_store(delay_reg, i32_type.const_int(instr.t_reg as u64, false));

        let delay_val = self.gep_load_delay_value(&format!("mfc0_{}_delay_val", self.count_uniq));
        self.builder.build_store(delay_val, cop_val);

        self.delay_slot_hazard = Some(|tb| tb.apply_load_delay_if_present());
    }
}

#[cfg(test)]
mod test {
    use crate::cpu::test::harness::TestHarness;

    #[test]
    fn jit_test_mtc0_mfc0() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::CpuState::default();

        let value = 10;

        th.load32(1, value);

        th.push_instr("mtc0", 1, 0, 1, 0, 0);
        th.push_instr("sll", 0, 0, 0, 0, 0);
        th.push_instr("mfc0", 1, 0, 2, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        println!("{:08x?}", state);
        assert_eq!(state.gpr[1], value);
    }

    #[test]
    fn jit_test_syscall() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::CpuState::default();

        th.push_instr("syscall", 0, 0, 0, 0, 0);
        th.push_instr("addiu", 0, 0, 1, 10, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        println!("{:08x?}", state);
        assert_eq!(state.gpr[0], 0);
        assert_eq!(state.pc, 0x8000_0080);
        assert_eq!(state.cop0_reg[super::cop0::Register::Epc as usize], 0x1000);
        assert_eq!(state.cop0_reg[super::cop0::Register::Cause as usize], 0x8 << 2);
    }

    #[test]
    fn jit_test_syscall_delay_slot() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::CpuState::default();

        th.push_instr("jr", 31, 0, 0, 0, 0);
        th.push_instr("syscall", 0, 0, 0, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        println!("{:08x?}", state);
        assert_eq!(state.pc, 0x8000_0080);
        assert_eq!(state.cop0_reg[super::cop0::Register::Epc as usize], 0x1000);
        assert_eq!(state.cop0_reg[super::cop0::Register::Cause as usize], (1 << 31) | (0x8 << 2));
    }

    #[test]
    fn jit_test_syscall_bev() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::CpuState::default();

        th.load32(1, 1 << 22);
        th.push_instr("mtc0", super::cop0::Register::Sr as u8, 0, 1, 0, 0);

        let syscall_pc = th.current_pc_head();
        th.push_instr("syscall", 0, 0, 0, 0, 0);
        th.push_instr("addiu", 0, 0, 2, 10, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        println!("{:08x?}", state);
        assert_eq!(state.gpr[1], 0);
        assert_eq!(state.pc, 0xbfc0_0180);
        assert_eq!(state.cop0_reg[super::cop0::Register::Epc as usize], syscall_pc);
        assert_eq!(state.cop0_reg[super::cop0::Register::Cause as usize], 0x8 << 2);
    }

    #[test]
    fn jit_test_break() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::CpuState::default();

        th.push_instr("break", 0, 0, 0, 0, 0);
        th.push_instr("addiu", 0, 0, 1, 10, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        println!("{:08x?}", state);
        assert_eq!(state.gpr[0], 0);
        assert_eq!(state.pc, 0x8000_0080);
        assert_eq!(state.cop0_reg[super::cop0::Register::Epc as usize], 0x1000);
        assert_eq!(state.cop0_reg[super::cop0::Register::Cause as usize], 0x9 << 2);
    }

    #[test]
    fn jit_test_break_delay_slot() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::CpuState::default();

        th.push_instr("jr", 31, 0, 0, 0, 0);
        th.push_instr("break", 0, 0, 0, 0, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        println!("{:08x?}", state);
        assert_eq!(state.pc, 0x8000_0080);
        assert_eq!(state.cop0_reg[super::cop0::Register::Epc as usize], 0x1000);
        assert_eq!(state.cop0_reg[super::cop0::Register::Cause as usize], (1 << 31) | (0x9 << 2));
    }

    #[test]
    fn jit_test_break_bev() {
        let mut th = TestHarness::default();
        let mut state = crate::cpu::CpuState::default();

        th.load32(1, 1 << 22);
        th.push_instr("mtc0", super::cop0::Register::Sr as u8, 0, 1, 0, 0);

        let syscall_pc = th.current_pc_head();
        th.push_instr("break", 0, 0, 0, 0, 0);
        th.push_instr("addiu", 0, 0, 2, 10, 0);
        th.finish();

        th.execute(&mut state).unwrap();

        println!("{:08x?}", state);
        assert_eq!(state.gpr[1], 0);
        assert_eq!(state.pc, 0xbfc0_0180);
        assert_eq!(state.cop0_reg[super::cop0::Register::Epc as usize], syscall_pc);
        assert_eq!(state.cop0_reg[super::cop0::Register::Cause as usize], 0x9 << 2);
    }
}
