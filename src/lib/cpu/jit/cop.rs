use crate::cpu::{decode::MipsCopInstr, opcode::MipsCopOperation};

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
}
