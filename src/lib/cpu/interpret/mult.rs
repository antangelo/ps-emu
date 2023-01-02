use super::{BusType, CpuState, MipsRInstr};

pub(super) fn interpret_mflo(
    instr: &MipsRInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    state.set_reg_val(instr.d_reg, state.lo);
    next_pc + 4
}

pub(super) fn interpret_mfhi(
    instr: &MipsRInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    state.set_reg_val(instr.d_reg, state.hi);
    next_pc + 4
}

pub(super) fn interpret_mtlo(
    instr: &MipsRInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    state.lo = state.get_reg_val(instr.s_reg);
    next_pc + 4
}

pub(super) fn interpret_mthi(
    instr: &MipsRInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    state.hi = state.get_reg_val(instr.s_reg);
    next_pc + 4
}

pub(super) fn interpret_mult(
    instr: &MipsRInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    let multiplier = state.get_reg_val(instr.s_reg) as i32;
    let multiplicand = state.get_reg_val(instr.t_reg) as i32;

    let product = (multiplier as i64) * (multiplicand as i64);
    state.lo = product as u32;
    state.hi = (product >> 32) as u32;

    next_pc + 4
}

pub(super) fn interpret_multu(
    instr: &MipsRInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    let multiplier = state.get_reg_val(instr.s_reg);
    let multiplicand = state.get_reg_val(instr.t_reg);

    let product = (multiplier as u64) * (multiplicand as u64);
    state.lo = product as u32;
    state.hi = (product >> 32) as u32;

    next_pc + 4
}

pub(super) fn interpret_divu(
    instr: &MipsRInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    let dividend = state.get_reg_val(instr.s_reg);
    let divisor = state.get_reg_val(instr.t_reg);

    if divisor == 0 {
        return next_pc + 4;
    }

    state.lo = ((dividend as u64) / (divisor as u64)) as u32;
    state.hi = ((dividend as u64) % (divisor as u64)) as u32;

    next_pc + 4
}

pub(super) fn interpret_div(
    instr: &MipsRInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    let dividend = state.get_reg_val(instr.s_reg) as i32;
    let divisor = state.get_reg_val(instr.t_reg) as i32;

    if divisor == 0 {
        return next_pc + 4;
    }

    state.lo = ((dividend as i64) / (divisor as i64)) as u32;
    state.hi = ((dividend as i64) % (divisor as i64)) as u32;

    next_pc + 4
}
