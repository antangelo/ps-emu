use super::decode;
use super::{DelaySlotArg, TranslationBlock};

impl<'ctx> TranslationBlock<'ctx> {
    fn branch_delay_slot_action<'a, 'b>(tb: &'a mut TranslationBlock<'b>) {
        let arg = tb.delay_slot_arg.as_ref().unwrap();
        let i32_type = tb.ctx.i32_type();

        let taken = tb
            .ctx
            .insert_basic_block_after(tb.func_block, &format!("beq_{}_taken", arg.count));

        {
            tb.builder.position_at_end(taken);

            // FIXME: This is stupid
            let immed_i = arg.immed as i16;
            let target = (immed_i as i32) * 4;

            let curr_pc_off = (tb.count_uniq * 4) as i32;
            let target_v = i32_type.const_int((target + curr_pc_off) as u64, true);

            let pc_ptr = tb.gep_pc(&format!("beq_{}", arg.count));
            let pc_val = tb
                .builder
                .build_load(pc_ptr, &format!("beq_{}_pc_val", arg.count));

            let next_pc = tb.builder.build_int_add(
                pc_val.into_int_value(),
                target_v,
                &format!("beq_{}_next_pc", arg.count),
            );
            tb.builder.build_store(pc_ptr, next_pc);
            tb.builder.build_return(None);
        }

        let not_taken = tb
            .ctx
            .insert_basic_block_after(taken, &format!("beq_{}_not_taken", arg.count));

        {
            tb.builder.position_at_end(not_taken);
            let target_v = i32_type.const_int((tb.count_uniq * 4 + 4) as u64, false);

            let pc_ptr = tb.gep_pc(&format!("beq_{}", arg.count));
            let pc_val = tb
                .builder
                .build_load(pc_ptr, &format!("beq_{}_pc_val", arg.count));

            let next_pc = tb.builder.build_int_add(
                pc_val.into_int_value(),
                target_v,
                &format!("beq_{}_next_pc", arg.count),
            );
            tb.builder.build_store(pc_ptr, next_pc);
            tb.builder.build_return(None);
        }

        tb.builder.position_at_end(tb.func_block);
        tb.builder
            .build_conditional_branch(arg.value.into_int_value(), taken, not_taken);
        tb.finalized = true;
    }

    pub(super) fn emit_bgtz(&mut self, instr: &decode::MipsIInstr) {
        let s_val = self.get_gpr_value(instr.s_reg, &format!("beq_{}", self.count_uniq));
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
