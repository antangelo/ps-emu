use object::{Endianness, Endian};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizedReadResult {
    Byte(u8),
    Word(u16),
    Dword(u32),
}

impl SizedReadResult {
    pub fn to_endian(&self, end: &Endianness) -> Self {
        match self {
            Self::Byte(u) => Self::Byte(*u),
            Self::Word(w) => Self::Word(end.write_u16(*w)),
            Self::Dword(d) => Self::Dword(end.write_u32(*d)),
        }
    }
}

pub trait BusDevice {
    fn validate(&mut self, base_addr: u32, size: u32);
    fn read(&mut self, addr: u32, size: u32) -> Result<SizedReadResult, MemAccessError>;
    fn write(&mut self, addr: u32, size: u32, value: u32) -> Result<(), MemAccessError>;
}

#[derive(Debug, Clone)]
pub enum MemAccessErrorType {
    NoEntry,
    NotInRange(u32, u32),
    ReadOnly,
    BadSize,
}

#[derive(Debug, Clone)]
pub struct MemAccessError {
    pub addr: u32,
    pub err: MemAccessErrorType,
}
