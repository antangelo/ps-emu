use super::mult;
use super::{BusType, CpuState, MipsFunction, MipsRInstr};

fn interpret_jr(
    instr: &MipsRInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    _next_pc: &u32,
) -> u32 {
    let target = if instr.s_reg != 0 {
        state.gpr[(instr.s_reg - 1) as usize]
    } else {
        0
    };

    target
}

fn interpret_jalr(
    instr: &MipsRInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    _next_pc: &u32,
) -> u32 {
    let target = state.get_reg_val(instr.s_reg);
    state.set_reg_val(instr.d_reg, state.pc + 8);

    target
}

fn interpret_sll(
    instr: &MipsRInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    let val = state.get_reg_val(instr.t_reg) << instr.shamt;
    state.set_reg_val(instr.d_reg, val as u32);
    next_pc + 4
}

fn interpret_srl(
    instr: &MipsRInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    let val = state.get_reg_val(instr.t_reg) >> instr.shamt;
    state.set_reg_val(instr.d_reg, val as u32);
    next_pc + 4
}

fn interpret_addu(
    instr: &MipsRInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    let val = state.get_reg_val(instr.t_reg) + state.get_reg_val(instr.s_reg);
    state.set_reg_val(instr.d_reg, val as u32);
    next_pc + 4
}

fn interpret_or(
    instr: &MipsRInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    let val = state.get_reg_val(instr.t_reg) | state.get_reg_val(instr.s_reg);
    state.set_reg_val(instr.d_reg, val as u32);
    next_pc + 4
}

pub(super) fn interpret_r_instr(
    instr: &MipsRInstr,
    bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    match instr.function {
        MipsFunction::Sll => interpret_sll(instr, bus, state, next_pc),
        MipsFunction::Srl => interpret_srl(instr, bus, state, next_pc),
        MipsFunction::Add => interpret_addu(instr, bus, state, next_pc),
        MipsFunction::AddU => interpret_addu(instr, bus, state, next_pc),
        MipsFunction::Or => interpret_or(instr, bus, state, next_pc),
        MipsFunction::Jr => interpret_jr(instr, bus, state, next_pc),
        MipsFunction::Jalr => interpret_jalr(instr, bus, state, next_pc),
        MipsFunction::Mfhi => mult::interpret_mfhi(instr, bus, state, next_pc),
        MipsFunction::Mflo => mult::interpret_mflo(instr, bus, state, next_pc),
        MipsFunction::Mthi => mult::interpret_mthi(instr, bus, state, next_pc),
        MipsFunction::Mtlo => mult::interpret_mtlo(instr, bus, state, next_pc),
        MipsFunction::Mult => mult::interpret_mult(instr, bus, state, next_pc),
        MipsFunction::MultU => mult::interpret_multu(instr, bus, state, next_pc),
        MipsFunction::Div => mult::interpret_div(instr, bus, state, next_pc),
        MipsFunction::DivU => mult::interpret_divu(instr, bus, state, next_pc),
        _ => panic!("Not implemented: {}", instr.function),
    }
}
