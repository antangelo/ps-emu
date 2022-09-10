#[derive(Debug, FromPrimitive)]
pub enum MipsOpcode {
    RegisterOp = 0x00,
    RegisterImm = 0x01,
    J = 0x02,
    Jal = 0x03,
    Beq = 0x04,
    Bne = 0x05,
    Blez = 0x06,
    Bgtz = 0x07,
    AddI = 0x08,
    AddIU = 0x09,
    SltI = 0x0a,
    SltIU = 0x0b,
    AndI = 0x0c,
    OrI = 0x0d,
    XorI = 0x0e,
    Lui = 0x0f,
    CoProc = 0x10,
    Lb = 0x20,
    Lh = 0x21,
    Lwl = 0x22,
    Lw = 0x23,
    Lbu = 0x24,
    Lhu = 0x25,
    Lwr = 0x26,
    Sb = 0x28,
    Sh = 0x29,
    Swl = 0x2a,
    Sw = 0x2b,
    Swr = 0x2e,
    LCoProc = 0x30,
    SwCoProc = 0x38,
}

impl MipsOpcode {
    fn to_str(&self) -> &str {
        match self {
            MipsOpcode::J => "j",
            MipsOpcode::Jal => "jal",
            MipsOpcode::Beq => "beq",
            MipsOpcode::Bne => "bne",
            MipsOpcode::Blez => "blez",
            MipsOpcode::Bgtz => "bgtz",
            MipsOpcode::AddI => "addi",
            MipsOpcode::AddIU => "addiu",
            MipsOpcode::SltI => "slti",
            MipsOpcode::SltIU => "sltiu",
            MipsOpcode::AndI => "andi",
            MipsOpcode::OrI => "ori",
            MipsOpcode::XorI => "xori",
            MipsOpcode::Lui => "lui",
            MipsOpcode::Lb => "lb",
            MipsOpcode::Lh => "lh",
            MipsOpcode::Lwl => "lwl",
            MipsOpcode::Lw => "lw",
            MipsOpcode::Lbu => "lbu",
            MipsOpcode::Lhu => "lhu",
            MipsOpcode::Lwr => "lwr",
            MipsOpcode::Sb => "sb",
            MipsOpcode::Sh => "sh",
            MipsOpcode::Swl => "swl",
            MipsOpcode::Sw => "sw",
            MipsOpcode::Swr => "swr",
            _ => "INVALID",
        }
    }
}

impl std::fmt::Display for MipsOpcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[derive(Debug, FromPrimitive)]
pub enum MipsFunction {
    Sll = 0x00,
    Srl = 0x02,
    Sra = 0x03,
    Sllv = 0x04,
    Srav = 0x05,
    Slrv = 0x06,
    Jr = 0x08,
    Jalr = 0x09,
    Syscall = 0x0a,
    Brk = 0x0d,
    Mfhi = 0x10,
    Mthi = 0x11,
    Mflo = 0x12,
    Mtlo = 0x13,
    Mult = 0x18,
    MultU = 0x19,
    Div = 0x1a,
    DivU = 0x1b,
    Add = 0x20,
    AddU = 0x21,
    Sub = 0x22,
    Subu = 0x23,
    Or = 0x25,
    Xor = 0x26,
    Nor = 0x27,
    And = 0x28,
    Sltu = 0x2b,
    Slt = 0x2c,
}

impl MipsFunction {
    fn to_str(&self) -> &str {
        match self {
            MipsFunction::Sll => "sll",
            MipsFunction::Srl => "srl",
            MipsFunction::Sra => "sra",
            MipsFunction::Sllv => "sllv",
            MipsFunction::Srav => "srav",
            MipsFunction::Slrv => "slrv",
            MipsFunction::Jr => "jr",
            MipsFunction::Jalr => "jalr",
            MipsFunction::Syscall => "syscall",
            MipsFunction::Brk => "break",
            MipsFunction::Mfhi => "mfhi",
            MipsFunction::Mthi => "mthi",
            MipsFunction::Mflo => "mflo",
            MipsFunction::Mtlo => "mtlo",
            MipsFunction::Mult => "mult",
            MipsFunction::MultU => "multu",
            MipsFunction::Div => "div",
            MipsFunction::DivU => "divu",
            MipsFunction::Add => "add",
            MipsFunction::AddU => "addu",
            MipsFunction::Sub => "sub",
            MipsFunction::Subu => "subu",
            MipsFunction::Or => "or",
            MipsFunction::Xor => "xor",
            MipsFunction::Nor => "nor",
            MipsFunction::And => "and",
            MipsFunction::Sltu => "sltu",
            MipsFunction::Slt => "slt",
        }
    }
}

impl std::fmt::Display for MipsFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[derive(Debug, FromPrimitive)]
pub enum MipsBranchSpecial {
    Bltz = 0x0,
    Bgez = 0x1,
    Bltzal = 0x10,
    Bgezal = 0x11,
    Invalid = 0x1f,
}

impl std::fmt::Display for MipsBranchSpecial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MipsBranchSpecial::Bltz => write!(f, "bltz"),
            MipsBranchSpecial::Bgez => write!(f, "begz"),
            MipsBranchSpecial::Bltzal => write!(f, "bltzal"),
            MipsBranchSpecial::Bgezal => write!(f, "bgezal"),
            _ => write!(f, "INVALID"),
        }
    }
}
