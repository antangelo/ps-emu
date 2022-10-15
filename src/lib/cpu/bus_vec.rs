use super::bus::{BusDevice, MemAccessError, MemAccessErrorType, SizedReadResult};

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
    fn validate(&mut self, _base_addr: u32, _size: u32) {}

    fn read(&mut self, addr: u32, size: u32) -> Result<SizedReadResult, MemAccessError> {
        for ent in &mut self.bus {
            if ent.addr <= addr && addr <= ent.addr + ent.size {
                return ent.device.read(addr - ent.addr, size);
            }
        }

        Err(MemAccessError {
            addr,
            err: MemAccessErrorType::NoEntry,
        })
    }

    fn write(&mut self, addr: u32, size: u32, value: u32) -> Result<(), MemAccessError> {
        for ent in &mut self.bus {
            if ent.addr <= addr && addr <= ent.addr + ent.size {
                return ent.device.write(addr - ent.addr, size, value);
            }
        }

        Err(MemAccessError {
            addr,
            err: MemAccessErrorType::NoEntry,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::cpu::bus::{BusDevice, SizedReadResult};
    struct SingleMemoryAddress {
        value: u32,
    }

    struct ReadOnly {
        value: u32,
    }

    impl super::BusDevice for SingleMemoryAddress {
        fn validate(&mut self, _base_addr: u32, _size: u32) {}
        fn read(
            &mut self,
            _addr: u32,
            _size: u32,
        ) -> Result<SizedReadResult, super::MemAccessError> {
            Ok(SizedReadResult::Dword(self.value))
        }

        fn write(
            &mut self,
            _addr: u32,
            _size: u32,
            value: u32,
        ) -> Result<(), super::MemAccessError> {
            self.value = value;
            Ok(())
        }
    }

    impl super::BusDevice for ReadOnly {
        fn validate(&mut self, _base_addr: u32, _size: u32) {}
        fn read(
            &mut self,
            _addr: u32,
            _size: u32,
        ) -> Result<SizedReadResult, super::MemAccessError> {
            Ok(SizedReadResult::Dword(self.value))
        }

        fn write(
            &mut self,
            addr: u32,
            _size: u32,
            _value: u32,
        ) -> Result<(), super::MemAccessError> {
            Err(super::MemAccessError {
                addr,
                err: super::MemAccessErrorType::ReadOnly,
            })
        }
    }

    #[test]
    fn bus_test_read() {
        let mut bus = super::VecBus::default();
        let mem_value = 2048;

        let dev = Box::new(SingleMemoryAddress { value: mem_value });
        bus.map(0x1000, 0x1000, dev);

        match bus.read(0x1004, 32) {
            Ok(v) => assert_eq!(SizedReadResult::Dword(mem_value), v),
            Err(e) => panic!("Memory error {:?}", e),
        }
    }

    #[test]
    fn bus_test_read_write() {
        let mut bus = super::VecBus::default();
        let mem_value = 2048;

        let dev = Box::new(SingleMemoryAddress { value: mem_value });
        bus.map(0x1000, 0x1000, dev);

        match bus.read(0x1004, 32) {
            Ok(v) => assert_eq!(SizedReadResult::Dword(mem_value), v),
            Err(e) => panic!("Memory error {:?}", e),
        }

        let new_value = 10;
        match bus.write(0x1004, 32, new_value) {
            Ok(_) => {}
            Err(e) => panic!("Memory error on write {:?}", e),
        }

        match bus.read(0x1004, 32) {
            Ok(v) => assert_eq!(SizedReadResult::Dword(new_value), v),
            Err(e) => panic!("Memory error {:?}", e),
        }
    }

    #[test]
    fn bus_test_read_only_write_err() {
        let mut bus = super::VecBus::default();
        let mem_value = 2048;

        let dev = Box::new(ReadOnly { value: mem_value });
        bus.map(0x1000, 0x1000, dev);

        match bus.read(0x1004, 32) {
            Ok(v) => assert_eq!(SizedReadResult::Dword(mem_value), v),
            Err(e) => panic!("Memory error {:?}", e),
        }

        match bus.write(0x1004, 32, 10) {
            Ok(_) => panic!("Write succeeded for RO memory region"),
            Err(_) => {}
        }

        match bus.read(0x1004, 32) {
            Ok(v) => assert_eq!(SizedReadResult::Dword(mem_value), v),
            Err(e) => panic!("Memory error {:?}", e),
        }
    }

    #[test]
    fn bus_test_unmapped_error() {
        let mut bus = super::VecBus::default();
        let mem_value = 2048;

        let dev = Box::new(SingleMemoryAddress { value: mem_value });
        bus.map(0x1000, 0x1000, dev);

        match bus.write(0x2004, 32, 10) {
            Ok(_) => panic!("Write succeeded for unmapped memory region"),
            Err(_) => {}
        }
    }

    #[test]
    #[should_panic]
    fn bus_test_overlapping_device_panics() {
        let mut bus = super::VecBus::default();
        let mem_value = 2048;

        let dev1 = Box::new(SingleMemoryAddress { value: mem_value });
        bus.map(0x1000, 0x1000, dev1);

        let dev2 = Box::new(SingleMemoryAddress { value: mem_value });
        bus.map(0x1000, 0x1000, dev2);
    }
}
