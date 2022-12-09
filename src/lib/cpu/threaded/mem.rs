use super::{BusDevice, BusType, CpuState, SizedReadResult, TbManager};

fn interpret_mem_read(
    s_reg: &u8,
    t_reg: &u8,
    immed: &u16,
    size: u32,
    bus: &mut BusType,
    state: &mut CpuState,
    sign_extend: bool,
) {
    let base = if *s_reg == 0 {
        0
    } else {
        state.gpr[(*s_reg - 1) as usize]
    };
    let addr = (base as i32 + *immed as i16 as i32) as u32;

    let read_result = bus.read(addr, size).unwrap();
    if *t_reg == 0 {
        return;
    }

    state.gpr[(*t_reg - 1) as usize] = match read_result {
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

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_lb(
    s_reg: u8,
    t_reg: u8,
    immed: u16,
    state: *mut CpuState,
    bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    interpret_mem_read(&s_reg, &t_reg, &immed, 8, &mut *bus, &mut *state, true);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_lbu(
    s_reg: u8,
    t_reg: u8,
    immed: u16,
    state: *mut CpuState,
    bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    interpret_mem_read(&s_reg, &t_reg, &immed, 8, &mut *bus, &mut *state, false);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_lw(
    s_reg: u8,
    t_reg: u8,
    immed: u16,
    state: *mut CpuState,
    bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    interpret_mem_read(&s_reg, &t_reg, &immed, 32, &mut *bus, &mut *state, false);
}

fn interpret_mem_write<'ctx>(
    s_reg: &u8,
    t_reg: &u8,
    immed: &u16,
    size: u32,
    bus: &mut BusType,
    state: &mut CpuState,
    mgr: &mut TbManager<'ctx>,
) {
    let base = if *s_reg == 0 {
        0
    } else {
        state.gpr[(*s_reg - 1) as usize]
    };
    let addr = (base as i32 + *immed as i16 as i32) as u32;

    let value = if *t_reg == 0 {
        0
    } else {
        state.gpr[(*t_reg - 1) as usize]
    };

    mgr.invalidate(addr);
    bus.write(addr, size, value).unwrap();
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_sb(
    s_reg: u8,
    t_reg: u8,
    immed: u16,
    state: *mut CpuState,
    bus: *mut BusType,
    mgr: *mut TbManager,
) {
    interpret_mem_write(&s_reg, &t_reg, &immed, 8, &mut *bus, &mut *state, &mut *mgr);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_sw(
    s_reg: u8,
    t_reg: u8,
    immed: u16,
    state: *mut CpuState,
    bus: *mut BusType,
    mgr: *mut TbManager,
) {
    interpret_mem_write(
        &s_reg,
        &t_reg,
        &immed,
        32,
        &mut *bus,
        &mut *state,
        &mut *mgr,
    );
}
