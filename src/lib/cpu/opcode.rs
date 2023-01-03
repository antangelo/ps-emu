#[derive(Debug, FromPrimitive, Clone, Copy)]
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
    pub(crate) fn cop_mem_from_str(s: &str) -> Option<(MipsOpcode, u8)> {
        match s {
            "lwc0" => Some((MipsOpcode::LCoProc, 0)),
            "lwc1" => Some((MipsOpcode::LCoProc, 1)),
            "lwc2" => Some((MipsOpcode::LCoProc, 2)),
            "lwc3" => Some((MipsOpcode::LCoProc, 3)),
            "swc0" => Some((MipsOpcode::SwCoProc, 0)),
            "swc1" => Some((MipsOpcode::SwCoProc, 1)),
            "swc2" => Some((MipsOpcode::SwCoProc, 2)),
            "swc3" => Some((MipsOpcode::SwCoProc, 3)),
            _ => None,
        }
    }

    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s {
            "j" => Some(MipsOpcode::J),
            "jal" => Some(MipsOpcode::Jal),
            "beq" => Some(MipsOpcode::Beq),
            "bne" => Some(MipsOpcode::Bne),
            "blez" => Some(MipsOpcode::Blez),
            "bgtz" => Some(MipsOpcode::Bgtz),
            "addi" => Some(MipsOpcode::AddI),
            "addiu" => Some(MipsOpcode::AddIU),
            "slti" => Some(MipsOpcode::SltI),
            "sltiu" => Some(MipsOpcode::SltIU),
            "andi" => Some(MipsOpcode::AndI),
            "ori" => Some(MipsOpcode::OrI),
            "xori" => Some(MipsOpcode::XorI),
            "lui" => Some(MipsOpcode::Lui),
            "lb" => Some(MipsOpcode::Lb),
            "lh" => Some(MipsOpcode::Lh),
            "lwl" => Some(MipsOpcode::Lwl),
            "lw" => Some(MipsOpcode::Lw),
            "lbu" => Some(MipsOpcode::Lbu),
            "lhu" => Some(MipsOpcode::Lhu),
            "lwr" => Some(MipsOpcode::Lwr),
            "sb" => Some(MipsOpcode::Sb),
            "sh" => Some(MipsOpcode::Sh),
            "swl" => Some(MipsOpcode::Swl),
            "sw" => Some(MipsOpcode::Sw),
            "swr" => Some(MipsOpcode::Swr),
            _ => None,
        }
    }

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

#[derive(Debug, FromPrimitive, Clone, Copy)]
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
    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s {
            "sll" => Some(MipsFunction::Sll),
            "srl" => Some(MipsFunction::Srl),
            "sra" => Some(MipsFunction::Sra),
            "sllv" => Some(MipsFunction::Sllv),
            "srav" => Some(MipsFunction::Srav),
            "slrv" => Some(MipsFunction::Slrv),
            "jr" => Some(MipsFunction::Jr),
            "jalr" => Some(MipsFunction::Jalr),
            "syscall" => Some(MipsFunction::Syscall),
            "break" => Some(MipsFunction::Brk),
            "mfhi" => Some(MipsFunction::Mfhi),
            "mthi" => Some(MipsFunction::Mthi),
            "mflo" => Some(MipsFunction::Mflo),
            "mtlo" => Some(MipsFunction::Mtlo),
            "mult" => Some(MipsFunction::Mult),
            "multu" => Some(MipsFunction::MultU),
            "div" => Some(MipsFunction::Div),
            "divu" => Some(MipsFunction::DivU),
            "add" => Some(MipsFunction::Add),
            "addu" => Some(MipsFunction::AddU),
            "sub" => Some(MipsFunction::Sub),
            "subu" => Some(MipsFunction::Subu),
            "or" => Some(MipsFunction::Or),
            "xor" => Some(MipsFunction::Xor),
            "nor" => Some(MipsFunction::Nor),
            "and" => Some(MipsFunction::And),
            "sltu" => Some(MipsFunction::Sltu),
            "slt" => Some(MipsFunction::Slt),
            _ => None,
        }
    }

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

#[derive(Debug, FromPrimitive, Copy, Clone)]
pub enum MipsCopOperation {
    MoveFrom = 0x0,
    ControlFrom = 0x2,
    MoveTo = 0x4,
    ControlTo = 0x6,
}

impl MipsCopOperation {
    pub(crate) fn from_str(s: &str) -> Option<(Self, u8)> {
        match s {
            "mfc0" => Some((Self::MoveFrom, 0)),
            "mfc1" => Some((Self::MoveFrom, 1)),
            "mfc2" => Some((Self::MoveFrom, 2)),
            "mfc3" => Some((Self::MoveFrom, 3)),
            "cfc0" => Some((Self::ControlFrom, 0)),
            "cfc1" => Some((Self::ControlFrom, 1)),
            "cfc2" => Some((Self::ControlFrom, 2)),
            "cfc3" => Some((Self::ControlFrom, 3)),
            "mtc0" => Some((Self::MoveTo, 0)),
            "mtc1" => Some((Self::MoveTo, 1)),
            "mtc2" => Some((Self::MoveTo, 2)),
            "mtc3" => Some((Self::MoveTo, 3)),
            "ctc0" => Some((Self::ControlTo, 0)),
            "ctc1" => Some((Self::ControlTo, 1)),
            "ctc2" => Some((Self::ControlTo, 2)),
            "ctc3" => Some((Self::ControlTo, 3)),
            _ => None,
        }
    }
}

impl std::fmt::Display for MipsCopOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MipsCopOperation::MoveFrom => write!(f, "mfc"),
            MipsCopOperation::ControlFrom => write!(f, "cfc"),
            MipsCopOperation::MoveTo => write!(f, "mtc"),
            MipsCopOperation::ControlTo => write!(f, "ctc"),
        }
    }
}
