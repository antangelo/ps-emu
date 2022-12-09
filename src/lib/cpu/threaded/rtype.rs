use super::opcode::MipsFunction;
use super::CpuState;
use super::{decode, DelaySlotArg};

use super::{BusType, TbManager, ThreadBlock};

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_jr(
    s_reg: u8,
    _t_reg: u8,
    _d_reg: u8,
    _shamt: u8,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
    _icount: u32,
) -> u32 {
    (*state).get_reg_val(s_reg)
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_jalr(
    s_reg: u8,
    _t_reg: u8,
    d_reg: u8,
    _shamt: u8,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
    icount: u32,
) -> u32 {
    let target = (*state).get_reg_val(s_reg);
    (*state).set_reg_val(d_reg, (*state).pc + 4 * icount + 8);

    target
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_sll(
    _s_reg: u8,
    t_reg: u8,
    d_reg: u8,
    shamt: u8,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    let val = (*state).get_reg_val(t_reg) << shamt;
    (*state).set_reg_val(d_reg, val as u32);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_srl(
    _s_reg: u8,
    t_reg: u8,
    d_reg: u8,
    shamt: u8,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    let val = (*state).get_reg_val(t_reg) >> shamt;
    (*state).set_reg_val(d_reg, val as u32);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_add(
    s_reg: u8,
    t_reg: u8,
    d_reg: u8,
    _shamt: u8,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    let val = (*state).get_reg_val(t_reg) + (*state).get_reg_val(s_reg);
    (*state).set_reg_val(d_reg, val as u32);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_or(
    s_reg: u8,
    t_reg: u8,
    d_reg: u8,
    _shamt: u8,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    let val = (*state).get_reg_val(t_reg) | (*state).get_reg_val(s_reg);
    (*state).set_reg_val(d_reg, val as u32);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_mflo(
    _s_reg: u8,
    _t_reg: u8,
    d_reg: u8,
    _shamt: u8,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    (*state).set_reg_val(d_reg, (*state).lo);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_mfhi(
    _s_reg: u8,
    _t_reg: u8,
    d_reg: u8,
    _shamt: u8,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    (*state).set_reg_val(d_reg, (*state).hi);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_mtlo(
    s_reg: u8,
    _t_reg: u8,
    _d_reg: u8,
    _shamt: u8,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    (*state).lo = (*state).get_reg_val(s_reg);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_mthi(
    s_reg: u8,
    _t_reg: u8,
    _d_reg: u8,
    _shamt: u8,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    (*state).hi = (*state).get_reg_val(s_reg);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_mult(
    s_reg: u8,
    t_reg: u8,
    _d_reg: u8,
    _shamt: u8,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    let multiplier = (*state).get_reg_val(s_reg) as i32;
    let multiplicand = (*state).get_reg_val(t_reg) as i32;

    let product = (multiplier as i64) * (multiplicand as i64);
    (*state).lo = product as u32;
    (*state).hi = (product >> 32) as u32;
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_multu(
    s_reg: u8,
    t_reg: u8,
    _d_reg: u8,
    _shamt: u8,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    let multiplier = (*state).get_reg_val(s_reg);
    let multiplicand = (*state).get_reg_val(t_reg);

    let product = (multiplier as u64) * (multiplicand as u64);
    (*state).lo = product as u32;
    (*state).hi = (product >> 32) as u32;
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_div(
    s_reg: u8,
    t_reg: u8,
    _d_reg: u8,
    _shamt: u8,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    let dividend = (*state).get_reg_val(s_reg) as i32;
    let divisor = (*state).get_reg_val(t_reg) as i32;

    if divisor == 0 {
        return;
    }

    (*state).lo = ((dividend as i64) / (divisor as i64)) as u32;
    (*state).hi = ((dividend as i64) % (divisor as i64)) as u32;
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_divu(
    s_reg: u8,
    t_reg: u8,
    _d_reg: u8,
    _shamt: u8,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    let dividend = (*state).get_reg_val(s_reg);
    let divisor = (*state).get_reg_val(t_reg);

    if divisor == 0 {
        return;
    }

    (*state).lo = ((dividend as u64) / (divisor as u64)) as u32;
    (*state).hi = ((dividend as u64) % (divisor as u64)) as u32;
}

impl<'ctx> ThreadBlock<'ctx> {
    pub(super) fn register_rtypes(
        &mut self,
        r_jmp_fn_type: &inkwell::types::FunctionType<'ctx>,
        r_fn_type: &inkwell::types::FunctionType<'ctx>,
    ) {
        self.register_rtype_jmp_fn(r_jmp_fn_type, MipsFunction::Jr, threaded_jr as usize);
        self.register_rtype_jmp_fn(r_jmp_fn_type, MipsFunction::Jalr, threaded_jalr as usize);

        self.register_rtype_fn(r_fn_type, MipsFunction::Sll, threaded_sll as usize);
        self.register_rtype_fn(r_fn_type, MipsFunction::Srl, threaded_srl as usize);
        self.register_rtype_fn(r_fn_type, MipsFunction::Or, threaded_or as usize);
        self.register_rtype_fn(r_fn_type, MipsFunction::Add, threaded_add as usize);
        self.register_rtype_fn(r_fn_type, MipsFunction::AddU, threaded_add as usize);

        self.register_rtype_fn(r_fn_type, MipsFunction::Mflo, threaded_mflo as usize);
        self.register_rtype_fn(r_fn_type, MipsFunction::Mfhi, threaded_mfhi as usize);
        self.register_rtype_fn(r_fn_type, MipsFunction::Mtlo, threaded_mtlo as usize);
        self.register_rtype_fn(r_fn_type, MipsFunction::Mthi, threaded_mthi as usize);
        self.register_rtype_fn(r_fn_type, MipsFunction::Mult, threaded_mult as usize);
        self.register_rtype_fn(r_fn_type, MipsFunction::MultU, threaded_multu as usize);
        self.register_rtype_fn(r_fn_type, MipsFunction::Div, threaded_div as usize);
        self.register_rtype_fn(r_fn_type, MipsFunction::DivU, threaded_divu as usize);
    }

    fn emit_rtype_jmp(&mut self, instr: &decode::MipsRInstr) {
        let i8_type = self.ctx.i8_type();
        let i32_type = self.ctx.i32_type();
        let s_reg = i8_type.const_int(instr.s_reg as u64, false);
        let t_reg = i8_type.const_int(instr.t_reg as u64, false);
        let d_reg = i8_type.const_int(instr.d_reg as u64, false);
        let shamt = i8_type.const_int(instr.shamt as u64, false);
        let icount = i32_type.const_int(self.icount as u64, false);

        let fn_name = format!("rtype_jmp_fn_{}", instr.function);
        let func = self
            .module
            .get_function(&fn_name)
            .expect(&format!("Not implemented: {}", instr.function));

        let cv = self
            .builder
            .build_call(
                func,
                &[
                    s_reg.into(),
                    t_reg.into(),
                    d_reg.into(),
                    shamt.into(),
                    self.state_arg.into(),
                    self.bus_arg.into(),
                    self.mgr_arg.into(),
                    icount.into(),
                ],
                &format!("rtype_jmp_call_{}", self.icount),
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
            let pc = tb.gep_pc(&format!("rtype_jmp_{}", arg.count));
            tb.builder.build_store(pc, arg.value);
            tb.builder.build_return(None);

            tb.finalized = true;
        });
    }

    fn emit_rtype_nojmp(&mut self, instr: &decode::MipsRInstr) {
        let i8_type = self.ctx.i8_type();
        let s_reg = i8_type.const_int(instr.s_reg as u64, false);
        let t_reg = i8_type.const_int(instr.t_reg as u64, false);
        let d_reg = i8_type.const_int(instr.d_reg as u64, false);
        let shamt = i8_type.const_int(instr.shamt as u64, false);

        let fn_name = format!("rtype_fn_{}", instr.function);
        let func = self
            .module
            .get_function(&fn_name)
            .expect(&format!("Not implemented: {}", instr.function));

        self.builder.build_call(
            func,
            &[
                s_reg.into(),
                t_reg.into(),
                d_reg.into(),
                shamt.into(),
                self.state_arg.into(),
                self.bus_arg.into(),
                self.mgr_arg.into(),
            ],
            &format!("rtype_call_{}", self.icount),
        );

        self.instr_finished_emitting();
    }

    pub(super) fn emit_rtype(&mut self, instr: &decode::MipsRInstr) {
        match instr.function {
            MipsFunction::Jr | MipsFunction::Jalr => self.emit_rtype_jmp(instr),
            MipsFunction::Srl
            | MipsFunction::Or
            | MipsFunction::Add
            | MipsFunction::AddU
            | MipsFunction::Mflo
            | MipsFunction::Mfhi
            | MipsFunction::Mtlo
            | MipsFunction::Mthi
            | MipsFunction::Mult
            | MipsFunction::MultU
            | MipsFunction::Div
            | MipsFunction::DivU
            | MipsFunction::Sll => self.emit_rtype_nojmp(instr),
            _ => panic!("Not implemented: {}", instr.function),
        }
    }
}
