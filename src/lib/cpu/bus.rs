pub trait BusDevice {
    fn validate(&mut self, base_addr: u32, size: u32);
    fn read(&mut self, addr: u32, size: u32) -> Result<u32, MemAccessError>;
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
