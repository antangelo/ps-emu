use super::opcode::{MipsBranchSpecial, MipsFunction, MipsOpcode};

#[derive(Debug)]
pub struct MipsRInstr {
    pub s_reg: u8,
    pub t_reg: u8,
    pub d_reg: u8,
    pub shamt: u8,
    pub function: MipsFunction,
}

#[derive(Debug)]
pub struct MipsIInstr {
    pub opcode: MipsOpcode,
    pub s_reg: u8,
    pub t_reg: u8,
    pub immediate: u16,
}

#[derive(Debug)]
pub struct MipsJInstr {
    pub opcode: MipsOpcode,
    pub target: u32,
}

#[derive(Debug)]
pub enum MipsInstr {
    RType(MipsRInstr),
    IType(MipsIInstr),
    JType(MipsJInstr),
    Invalid,
}

fn mips_decode_rtype(instr_raw: u32) -> MipsInstr {
    let s_reg = ((instr_raw >> 21) & 0x1f) as u8;
    let t_reg = ((instr_raw >> 16) & 0x1f) as u8;
    let d_reg = ((instr_raw >> 11) & 0x1f) as u8;
    let shamt = ((instr_raw >> 6) & 0x1f) as u8;
    let func = num::FromPrimitive::from_u32(instr_raw & 0x3f);

    if let Some(function) = func {
        return MipsInstr::RType(MipsRInstr {
            s_reg,
            t_reg,
            d_reg,
            shamt,
            function,
        });
    }

    MipsInstr::Invalid
}

fn mips_decode_itype(opcode: MipsOpcode, instr_raw: u32) -> MipsInstr {
    let s_reg = ((instr_raw >> 21) & 0x1f) as u8;
    let t_reg = ((instr_raw >> 16) & 0x1f) as u8;
    let immediate = (instr_raw & 0xffff) as u16;

    MipsInstr::IType(MipsIInstr {
        opcode,
        s_reg,
        t_reg,
        immediate,
    })
}

fn mips_decode_jtype(opcode: MipsOpcode, instr_raw: u32) -> MipsInstr {
    let target = instr_raw & 0x03ffffff;

    MipsInstr::JType(MipsJInstr { opcode, target })
}

fn mips_decode_opcode(opcode: MipsOpcode, instr_raw: u32) -> MipsInstr {
    match opcode {
        MipsOpcode::RegisterOp => mips_decode_rtype(instr_raw),
        MipsOpcode::J | MipsOpcode::Jal => mips_decode_jtype(opcode, instr_raw),
        _ => mips_decode_itype(opcode, instr_raw),
    }
}

pub fn mips_decode(instr_raw: u32) -> MipsInstr {
    let opcode = num::FromPrimitive::from_u32(instr_raw >> 26);

    match opcode {
        Some(op) => mips_decode_opcode(op, instr_raw),
        None => MipsInstr::Invalid,
    }
}

fn mips_encode_rtype(instr: &MipsRInstr) -> u32 {
    let mut res: u32 = instr.function as u32;
    res |= (instr.shamt as u32) << 6;
    res |= (instr.d_reg as u32) << 11;
    res |= (instr.t_reg as u32) << 16;
    res |= (instr.s_reg as u32) << 21;

    res
}

fn mips_encode_itype(instr: &MipsIInstr) -> u32 {
    let mut res: u32 = (instr.opcode as u32) << 26;
    res |= (instr.s_reg as u32) << 21;
    res |= (instr.t_reg as u32) << 16;
    res |= instr.immediate as u32;

    res
}

fn mips_encode_jtype(instr: &MipsJInstr) -> u32 {
    let mut res: u32 = (instr.opcode as u32) << 26;
    res |= instr.target;

    res
}

pub fn mips_encode(instr: &MipsInstr) -> Option<u32> {
    match instr {
        MipsInstr::RType(r) => Some(mips_encode_rtype(r)),
        MipsInstr::IType(i) => Some(mips_encode_itype(i)),
        MipsInstr::JType(j) => Some(mips_encode_jtype(j)),
        MipsInstr::Invalid => None,
    }
}

pub fn mips_encode_str(istr: &str, d: u8, s: u8, t: u8, imm: u16, tgt: u32) -> Option<u32> {
    if let Some(op) = MipsOpcode::from_str(istr) {
        let instr = match op {
            MipsOpcode::J | MipsOpcode::Jal => MipsInstr::JType(MipsJInstr {
                opcode: op,
                target: tgt,
            }),
            _ => MipsInstr::IType(MipsIInstr {
                opcode: op,
                s_reg: s,
                t_reg: t,
                immediate: imm,
            }),
        };

        return mips_encode(&instr);
    }

    if let Some(function) = super::opcode::MipsFunction::from_str(istr) {
        let instr = MipsRInstr {
            s_reg: s,
            d_reg: d,
            t_reg: t,
            shamt: imm as u8,
            function,
        };

        return mips_encode(&MipsInstr::RType(instr));
    }

    None
}

impl std::fmt::Display for MipsRInstr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.function {
            MipsFunction::Sll | MipsFunction::Srl | MipsFunction::Sra => write!(
                f,
                "{} ${}, ${}, {}",
                self.function, self.d_reg, self.t_reg, self.shamt
            ),
            MipsFunction::Jr | MipsFunction::Jalr => write!(f, "{} ${}", self.function, self.s_reg),
            _ => write!(
                f,
                "{} ${}, ${}, ${}",
                self.function, self.d_reg, self.s_reg, self.t_reg
            ),
        }
    }
}

impl std::fmt::Display for MipsIInstr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.opcode {
            MipsOpcode::RegisterImm => {
                let special_op =
                    num::FromPrimitive::from_u8(self.t_reg).unwrap_or(MipsBranchSpecial::Invalid);
                write!(
                    f,
                    "{} ${}, {}",
                    special_op, self.s_reg, self.immediate as i16
                )
            }
            MipsOpcode::Lb
            | MipsOpcode::Lh
            | MipsOpcode::Lw
            | MipsOpcode::Lwl
            | MipsOpcode::Lbu
            | MipsOpcode::Lhu
            | MipsOpcode::Lwr
            | MipsOpcode::Sb
            | MipsOpcode::Sh
            | MipsOpcode::Sw
            | MipsOpcode::Swl
            | MipsOpcode::Swr => write!(
                f,
                "{} ${}, {}(${})",
                self.opcode, self.t_reg, self.immediate, self.s_reg
            ),
            _ => write!(
                f,
                "{} ${}, ${}, {}",
                self.opcode, self.t_reg, self.s_reg, self.immediate as i16
            ),
        }
    }
}

impl std::fmt::Display for MipsJInstr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:#x}", self.opcode, self.target << 2)
    }
}

impl std::fmt::Display for MipsInstr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MipsInstr::RType(r) => r.fmt(f),
            MipsInstr::IType(i) => i.fmt(f),
            MipsInstr::JType(j) => j.fmt(f),
            MipsInstr::Invalid => write!(f, "INVALID"),
        }
    }
}
