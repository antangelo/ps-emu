use super::{BusType, CpuState, MipsIInstr};

pub(super) fn interpret_bne(
    instr: &MipsIInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    _next_pc: &u32,
) -> u32 {
    let s_val = state.get_reg_val(instr.s_reg);
    let t_val = state.get_reg_val(instr.t_reg);

    let target = if s_val != t_val {
        (state.pc as i32 + (instr.immediate as i16 as i32) * 4 + 4) as u32
    } else {
        state.pc + 8
    };

    target
}

pub(super) fn interpret_beq(
    instr: &MipsIInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    _next_pc: &u32,
) -> u32 {
    let s_val = state.get_reg_val(instr.s_reg);
    let t_val = state.get_reg_val(instr.t_reg);

    let target = if s_val == t_val {
        (state.pc as i32 + (instr.immediate as i16 as i32) * 4 + 4) as u32
    } else {
        state.pc + 8
    };

    target
}

pub(super) fn interpret_bgtz(
    instr: &MipsIInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    _next_pc: &u32,
) -> u32 {
    let s_val = state.get_reg_val(instr.s_reg);

    let target = if s_val > 0 {
        (state.pc as i32 + (instr.immediate as i16 as i32) * 4 + 4) as u32
    } else {
        state.pc + 8
    };

    target
}
