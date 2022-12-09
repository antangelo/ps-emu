use super::{BusType, CpuState, MipsIInstr};
use crate::cpu::bus::{BusDevice, SizedReadResult};

fn interpret_mem_read(
    instr: &MipsIInstr,
    size: u32,
    bus: &mut BusType,
    state: &mut CpuState,
    sign_extend: bool,
) {
    let base = if instr.s_reg == 0 {
        0
    } else {
        state.gpr[(instr.s_reg - 1) as usize]
    };
    let addr = (base as i32 + instr.immediate as i16 as i32) as u32;
    let reg = instr.t_reg;

    let read_result = bus.read(addr, size).unwrap();
    if reg == 0 {
        return;
    }

    state.gpr[(reg - 1) as usize] = match read_result {
        SizedReadResult::Byte(b) => {
            if sign_extend {
                b as i8 as u32
            } else {
                b as u32
            }
        }
        SizedReadResult::Word(w) => {
            if sign_extend {
                w as i16 as u32
            } else {
                w as u32
            }
        }
        SizedReadResult::Dword(d) => d,
    };
}

pub(super) fn interpret_lb(
    instr: &MipsIInstr,
    bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    interpret_mem_read(instr, 8, bus, state, true);
    next_pc + 4
}

pub(super) fn interpret_lbu(
    instr: &MipsIInstr,
    bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    interpret_mem_read(instr, 8, bus, state, false);
    next_pc + 4
}

pub(super) fn interpret_lw(
    instr: &MipsIInstr,
    bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    interpret_mem_read(instr, 32, bus, state, false);
    next_pc + 4
}

fn interpret_mem_write(instr: &MipsIInstr, size: u32, bus: &mut BusType, state: &mut CpuState) {
    let base = if instr.s_reg == 0 {
        0
    } else {
        state.gpr[(instr.s_reg - 1) as usize]
    };
    let addr = (base as i32 + instr.immediate as i16 as i32) as u32;
    let reg = instr.t_reg;

    let value = if reg == 0 {
        0
    } else {
        state.gpr[(reg - 1) as usize]
    };

    bus.write(addr, size, value).unwrap();
}

pub(super) fn interpret_sw(
    instr: &MipsIInstr,
    bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    interpret_mem_write(instr, 32, bus, state);
    next_pc + 4
}

pub(super) fn interpret_sb(
    instr: &MipsIInstr,
    bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    interpret_mem_write(instr, 8, bus, state);
    next_pc + 4
}
