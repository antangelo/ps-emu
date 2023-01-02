use super::bus::{BusDevice, SizedReadResult};
use super::bus_vec::VecBus;
use super::decode::{MipsIInstr, MipsInstr, MipsJInstr, MipsRInstr};
use super::opcode::{MipsFunction, MipsOpcode};
use super::CpuState;

type BusType = VecBus;

mod branch;
mod jtype;
mod mem;
mod mult;
mod rtype;

fn interpret_addiu(
    instr: &MipsIInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    let val = state.get_reg_val(instr.s_reg) as i32 + instr.immediate as i16 as i32;
    state.set_reg_val(instr.t_reg, val as u32);

    next_pc + 4
}

fn interpret_sltiu(
    instr: &MipsIInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    let s_val = state.get_reg_val(instr.s_reg);
    let val = if s_val < instr.immediate as u32 { 1 } else { 0 };

    state.set_reg_val(instr.t_reg, val as u32);

    next_pc + 4
}

fn interpret_slti(
    instr: &MipsIInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    let s_val = state.get_reg_val(instr.s_reg) as i32;
    let val = if s_val < instr.immediate as i16 as i32 {
        1
    } else {
        0
    };

    state.set_reg_val(instr.t_reg, val as u32);

    next_pc + 4
}

fn interpret_ori(
    instr: &MipsIInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    let val = state.get_reg_val(instr.s_reg) | instr.immediate as u32;
    state.set_reg_val(instr.t_reg, val as u32);

    next_pc + 4
}

fn interpret_lui(
    instr: &MipsIInstr,
    _bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    let val = (instr.immediate as u32) << 16;
    state.set_reg_val(instr.t_reg, val as u32);

    next_pc + 4
}

fn interpret_i_instr(
    instr: &MipsIInstr,
    bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> u32 {
    match instr.opcode {
        MipsOpcode::AddIU => interpret_addiu(instr, bus, state, next_pc),
        MipsOpcode::SltIU => interpret_sltiu(instr, bus, state, next_pc),
        MipsOpcode::SltI => interpret_slti(instr, bus, state, next_pc),
        MipsOpcode::OrI => interpret_ori(instr, bus, state, next_pc),
        MipsOpcode::Lui => interpret_lui(instr, bus, state, next_pc),
        MipsOpcode::Lb => mem::interpret_lb(instr, bus, state, next_pc),
        MipsOpcode::Lbu => mem::interpret_lbu(instr, bus, state, next_pc),
        MipsOpcode::Lw => mem::interpret_lw(instr, bus, state, next_pc),
        MipsOpcode::Sw => mem::interpret_sw(instr, bus, state, next_pc),
        MipsOpcode::Sb => mem::interpret_sb(instr, bus, state, next_pc),
        MipsOpcode::Bne => branch::interpret_bne(instr, bus, state, next_pc),
        MipsOpcode::Beq => branch::interpret_beq(instr, bus, state, next_pc),
        MipsOpcode::Bgtz => branch::interpret_bgtz(instr, bus, state, next_pc),
        _ => panic!("Not implemented: {} @ {:08x}", instr.opcode, state.pc),
    }
}

fn interpret_instruction(
    bus: &mut BusType,
    state: &mut CpuState,
    next_pc: &u32,
) -> Result<u32, String> {
    let read_result = bus
        .read(state.pc, 32)
        .map_err(|_| format!("Failed to read instr at pc {:08x}", state.pc))?;
    if let SizedReadResult::Dword(instr_raw) = read_result {
        let instr = super::decode::mips_decode(instr_raw);

        let delay_slot_action = match instr {
            MipsInstr::RType(r) => rtype::interpret_r_instr(&r, bus, state, next_pc),
            MipsInstr::IType(i) => interpret_i_instr(&i, bus, state, next_pc),
            MipsInstr::JType(j) => jtype::interpret_j_instr(&j, bus, state),
            _ => state.pc + 8,
        };

        state.pc = *next_pc;
        Ok(delay_slot_action)
    } else {
        panic!(
            "Read size of 32 didn't return dword, instead have {:?}",
            read_result
        );
    }
}

pub fn execute(bus: &mut BusType, state: &mut CpuState) -> Result<(), String> {
    let mut icount: u64 = 0;
    let mut icount_tot = 0;
    let now = std::time::Instant::now();
    let mut prev_elapsed: u128 = 0;

    let mut mips_avg: f64 = 0.0;
    let mut mips_min: f64 = f64::MAX;
    let mut mips_max: f64 = 0.0;
    let mut mips_avg_count: u128 = 0;

    let mut next_pc = state.pc + 4;
    let mut prev_pc: u32;

    let timing_scale = 1_000;

    loop {
        prev_pc = state.pc;
        next_pc = interpret_instruction(bus, state, &next_pc)?;
        icount += 1;

        if icount > timing_scale {
            let elapsed_micros_tot = now.elapsed().as_micros();
            let elapsed_micros = elapsed_micros_tot - prev_elapsed;
            prev_elapsed = elapsed_micros_tot;
            let elapsed = (elapsed_micros as f64) / 1_000_000.0;
            let mips = (icount as f64) / elapsed / 1_000_000.0;
            mips_min = f64::min(mips_min, mips);
            mips_max = f64::max(mips_max, mips);
            mips_avg += mips;
            mips_avg_count += 1;

            icount_tot += icount;
            icount = 0;
        }

        //println!("{:08x} {:08x}", state.pc, next_pc);
        if next_pc == prev_pc {
            break;
        }
    }

    let elapsed_micros = now.elapsed().as_micros();
    let elapsed = (elapsed_micros as f64) / 1_000_000.0;

    println!("CpuState: {:x?}", state);
    println!("elapsed time: {}", elapsed);
    println!("icount: {}", icount_tot + icount);
    println!("MIPS (average): {}", mips_avg / (mips_avg_count as f64));
    println!("MIPS (min): {}", mips_min);
    println!("MIPS (max): {}", mips_max);

    Ok(())
}
