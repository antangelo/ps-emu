use super::{BusType, CpuState, MipsJInstr, MipsOpcode};

fn interpret_j(instr: &MipsJInstr, _bus: &mut BusType, _state: &mut CpuState) -> u32 {
    let target = instr.target;
    target << 2
}

fn interpret_jal(instr: &MipsJInstr, _bus: &mut BusType, state: &mut CpuState) -> u32 {
    let target = instr.target;
    state.gpr[30] = state.pc + 8;
    target << 2
}

pub(super) fn interpret_j_instr(
    instr: &MipsJInstr,
    bus: &mut BusType,
    state: &mut CpuState,
) -> u32 {
    match instr.opcode {
        MipsOpcode::J => interpret_j(instr, bus, state),
        MipsOpcode::Jal => interpret_jal(instr, bus, state),
        _ => panic!("Not J type instruction: {}", instr.opcode),
    }
}
