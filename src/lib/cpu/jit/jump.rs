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
