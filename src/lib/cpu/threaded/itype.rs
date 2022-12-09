use super::mem;
use super::opcode::MipsOpcode;
use super::{decode, DelaySlotArg};

use super::{BusType, CpuState, TbManager, ThreadBlock};

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_bne(
    s_reg: u8,
    t_reg: u8,
    immed: u16,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
    icount: u32,
) -> u32 {
    let s_val = (*state).get_reg_val(s_reg);
    let t_val = (*state).get_reg_val(t_reg);

    let pc = (*state).pc + 4 * icount;

    let target = if s_val != t_val {
        (pc as i32 + (immed as i16 as i32) * 4 + 4) as u32
    } else {
        pc + 8
    };

    target
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_beq(
    s_reg: u8,
    t_reg: u8,
    immed: u16,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
    icount: u32,
) -> u32 {
    let s_val = (*state).get_reg_val(s_reg);
    let t_val = (*state).get_reg_val(t_reg);

    let pc = (*state).pc + 4 * icount;

    let target = if s_val == t_val {
        (pc as i32 + (immed as i16 as i32) * 4 + 4) as u32
    } else {
        pc + 8
    };

    target
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_bgtz(
    s_reg: u8,
    _t_reg: u8,
    immed: u16,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
    icount: u32,
) -> u32 {
    let s_val = (*state).get_reg_val(s_reg);

    let pc = (*state).pc + 4 * icount;

    let target = if s_val > 0 {
        (pc as i32 + (immed as i16 as i32) * 4 + 4) as u32
    } else {
        pc + 8
    };

    target
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_addi(
    s_reg: u8,
    t_reg: u8,
    immed: u16,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    let val = (*state).get_reg_val(s_reg) + immed as i16 as i32 as u32;
    (*state).set_reg_val(t_reg, val);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_sltiu(
    s_reg: u8,
    t_reg: u8,
    immed: u16,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    let s_val = (*state).get_reg_val(s_reg);
    let val = if s_val < immed as u32 { 1 } else { 0 };

    (*state).set_reg_val(t_reg, val as u32);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_slti(
    s_reg: u8,
    t_reg: u8,
    immed: u16,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    let s_val = (*state).get_reg_val(s_reg) as i32;
    let val = if s_val < immed as i16 as i32 { 1 } else { 0 };

    (*state).set_reg_val(t_reg, val as u32);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_ori(
    s_reg: u8,
    t_reg: u8,
    immed: u16,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    let val = (*state).get_reg_val(s_reg) | (immed as u32);
    (*state).set_reg_val(t_reg, val);
}

#[no_mangle]
pub(super) unsafe extern "C" fn threaded_lui(
    _s_reg: u8,
    t_reg: u8,
    immed: u16,
    state: *mut CpuState,
    _bus: *mut BusType,
    _mgr: *mut TbManager,
) {
    let val = (immed as u32) << 16;
    (*state).set_reg_val(t_reg, val);
}

impl<'ctx> ThreadBlock<'ctx> {
    pub(super) fn register_itypes(
        &mut self,
        i_jmp_fn_type: &inkwell::types::FunctionType<'ctx>,
        i_fn_type: &inkwell::types::FunctionType<'ctx>,
    ) {
        self.register_itype_jmp_fn(&i_jmp_fn_type, MipsOpcode::Bne, threaded_bne as usize);
        self.register_itype_jmp_fn(&i_jmp_fn_type, MipsOpcode::Beq, threaded_beq as usize);
        self.register_itype_jmp_fn(&i_jmp_fn_type, MipsOpcode::Bgtz, threaded_bgtz as usize);

        //self.register_itype_fn(&i_fn_type, MipsOpcode::AddI, threaded_addi as usize);
        self.register_itype_fn(&i_fn_type, MipsOpcode::AddIU, threaded_addi as usize);
        self.register_itype_fn(&i_fn_type, MipsOpcode::SltI, threaded_slti as usize);
        self.register_itype_fn(&i_fn_type, MipsOpcode::SltIU, threaded_sltiu as usize);
        self.register_itype_fn(&i_fn_type, MipsOpcode::OrI, threaded_ori as usize);
        self.register_itype_fn(&i_fn_type, MipsOpcode::Lui, threaded_lui as usize);

        self.register_itype_fn(&i_fn_type, MipsOpcode::Lb, mem::threaded_lb as usize);
        self.register_itype_fn(&i_fn_type, MipsOpcode::Lbu, mem::threaded_lbu as usize);
        self.register_itype_fn(&i_fn_type, MipsOpcode::Lw, mem::threaded_lw as usize);
        self.register_itype_fn(&i_fn_type, MipsOpcode::Sb, mem::threaded_sb as usize);
        self.register_itype_fn(&i_fn_type, MipsOpcode::Sw, mem::threaded_sw as usize);
    }

    fn emit_itype_jmp(&mut self, instr: &decode::MipsIInstr) {
        let i8_type = self.ctx.i8_type();
        let i16_type = self.ctx.i16_type();
        let i32_type = self.ctx.i32_type();
        let s_reg = i8_type.const_int(instr.s_reg as u64, false);
        let t_reg = i8_type.const_int(instr.t_reg as u64, false);
        let immed = i16_type.const_int(instr.immediate as u64, false);
        let icount = i32_type.const_int(self.icount as u64, false);

        let fn_name = format!("itype_jmp_fn_{}", instr.opcode);
        let func = self
            .module
            .get_function(&fn_name)
            .expect(&format!("Not implemented: {}", instr.opcode));

        let cv = self
            .builder
            .build_call(
                func,
                &[
                    s_reg.into(),
                    t_reg.into(),
                    immed.into(),
                    self.state_arg.into(),
                    self.bus_arg.into(),
                    self.mgr_arg.into(),
                    icount.into(),
                ],
                &format!("itype_jmp_call_{}", self.icount),
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
            let pc = tb.gep_pc(&format!("itype_jmp_{}", arg.count));
            tb.builder.build_store(pc, arg.value);
            tb.builder.build_return(None);

            tb.finalized = true;
        });
    }

    fn emit_itype_nojmp(&mut self, instr: &decode::MipsIInstr) {
        let i8_type = self.ctx.i8_type();
        let i16_type = self.ctx.i16_type();
        let s_reg = i8_type.const_int(instr.s_reg as u64, false);
        let t_reg = i8_type.const_int(instr.t_reg as u64, false);
        let immed = i16_type.const_int(instr.immediate as u64, false);

        let fn_name = format!("itype_fn_{}", instr.opcode);
        let func = self
            .module
            .get_function(&fn_name)
            .expect(&format!("Not implemented: {}", instr.opcode));

        self.builder.build_call(
            func,
            &[
                s_reg.into(),
                t_reg.into(),
                immed.into(),
                self.state_arg.into(),
                self.bus_arg.into(),
                self.mgr_arg.into(),
            ],
            &format!("itype_call_{}", self.icount),
        );

        self.instr_finished_emitting();
    }

    pub(super) fn emit_itype(&mut self, instr: &decode::MipsIInstr) {
        match instr.opcode {
            MipsOpcode::Beq | MipsOpcode::Bne | MipsOpcode::Bgtz => self.emit_itype_jmp(instr),
            MipsOpcode::AddI
            | MipsOpcode::AddIU
            | MipsOpcode::SltI
            | MipsOpcode::SltIU
            | MipsOpcode::OrI
            | MipsOpcode::Lui
            | MipsOpcode::Lb
            | MipsOpcode::Lbu
            | MipsOpcode::Lw
            | MipsOpcode::Sb
            | MipsOpcode::Sw => self.emit_itype_nojmp(instr),
            _ => panic!("Not implemented: {}", instr.opcode),
        }
    }
}
