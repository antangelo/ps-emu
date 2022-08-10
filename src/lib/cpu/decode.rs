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
