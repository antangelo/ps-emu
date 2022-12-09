use super::opcode::MipsOpcode;
use super::CpuState;
use super::{decode, BusType, DelaySlotArg, TbManager, ThreadBlock};

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_j(
    target: u32,
    _state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
    _icount: u32,
) -> u32 {
    target << 2
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_jal(
    target: u32,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
    icount: u32,
) -> u32 {
    (*state).set_reg_val(31, (*state).pc + 4 * icount + 8);
    target << 2
}

impl<'ctx> ThreadBlock<'ctx> {
    pub(super) fn register_jtypes(&mut self, j_fn_type: &inkwell::types::FunctionType<'ctx>) {
        self.register_jtype_fn(&j_fn_type, MipsOpcode::J, threaded_j as usize);
        self.register_jtype_fn(&j_fn_type, MipsOpcode::Jal, threaded_jal as usize);
    }

    pub(super) fn emit_jtype(&mut self, instr: &decode::MipsJInstr) {
        let i32_type = self.ctx.i32_type();
        let target = i32_type.const_int(instr.target as u64, false);
        let icount = i32_type.const_int(self.icount as u64, false);

        let fn_name = format!("jtype_fn_{}", instr.opcode);
        let func = self
            .module
            .get_function(&fn_name)
            .expect(&format!("Not implemented: {}", instr.opcode));

        let cv = self
            .builder
            .build_call(
                func,
                &[
                    target.into(),
                    self.state_arg.into(),
                    self.bus_arg.into(),
                    self.mgr_arg.into(),
                    icount.into(),
                ],
                &format!("jtype_call_{}", self.icount),
            )
            .try_as_basic_value()
            .left()
            .unwrap();

        self.instr_finished_emitting();

        self.delay_slot_arg = Some(DelaySlotArg {
            count: self.icount as u64,
            value: cv,
        });

        self.delay_slot_fn = Some(|tb| {
            let arg = tb.delay_slot_arg.as_ref().unwrap();
            let pc = tb.gep_pc(&format!("jtype_{}", arg.count));
            tb.builder.build_store(pc, arg.value);
            tb.builder.build_return(None);

            tb.finalized = true;
        });
    }
}
