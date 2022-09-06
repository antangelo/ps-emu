use super::bus::{MemAccessError, MemAccessErrorType, BusDevice};

struct BusEntry {
    addr: u32,
    size: u32,
    device: Box<dyn BusDevice>,
}

pub struct VecBus {
    pub endianness: object::Endianness,
    bus: Vec<BusEntry>,
}

impl Default for VecBus {
    fn default() -> Self {
        Self {
            endianness: object::Endianness::Little,
            bus: Vec::default(),
        }
    }
}

impl VecBus {
    pub fn map(&mut self, addr: u32, size: u32, mut device: Box<dyn BusDevice>) {
        device.validate(addr, size);

        for ent in &self.bus {
            if addr >= ent.addr && addr < ent.addr + ent.size {
                panic!(
                    "Overlapping bus entry at addrs ({} to {}) and ({} to {})",
                    addr, size, ent.addr, ent.size
                );
            }
        }

        self.bus.push(BusEntry { addr, size, device });
    }
}

impl BusDevice for VecBus {
    fn validate(&mut self, _base_addr: u32, _size: u32) {
    }

    fn read(&mut self, addr: u32, size: u32) -> Result<u32, MemAccessError> {
        for ent in &mut self.bus {
            if ent.addr <= addr && addr <= ent.addr + ent.size {
                return ent.device.read(addr, size);
            }
        }

        Err(MemAccessError{ addr, err: MemAccessErrorType::NoEntry })
    }

    fn write(&mut self, addr: u32, size: u32, value: u32) -> Result<(), MemAccessError> {
        for ent in &mut self.bus {
            if ent.addr <= addr && addr <= ent.addr + ent.size {
                return ent.device.write(addr, size, value);
            }
        }

        Err(MemAccessError{ addr, err: MemAccessErrorType::NoEntry })
    }
}
