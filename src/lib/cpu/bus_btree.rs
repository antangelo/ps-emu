use object::Endian;
use super::bus::{MemAccessError, MemAccessErrorType, BusDevice};

struct BusEntry {
    addr: u32,
    size: u32,
    device: Box<dyn BusDevice>,
}

pub struct BTreeBus {
    pub endianness: object::Endianness,
    bus: std::collections::BTreeMap<u32, BusEntry>,
}

impl Default for BTreeBus {
    fn default() -> Self {
        Self {
            endianness: object::Endianness::Little,
            bus: std::collections::BTreeMap::default(),
        }
    }
}

impl BTreeBus {
    pub fn map(&mut self, addr: u32, size: u32, mut device: Box<dyn BusDevice>) {
        device.validate(addr, size);
        if let Some((a, ent)) = self.bus.range(..=addr).next_back() {
            if addr >= *a && addr < *a + ent.size {
                panic!(
                    "Overlapping bus entry at addrs ({} to {}) and ({} to {})",
                    addr, size, *a, ent.size
                );
            }
        }

        if let Some(_) = self.bus.insert(addr, BusEntry { addr, size, device }) {
            panic!("Bus insert overwrote entry");
        }
    }

    fn bus_lookup(&mut self, addr: u32) -> Result<&mut BusEntry, MemAccessError> {
        if let Some((a, ent)) = self.bus.range_mut(..=addr).next_back() {
            if addr < *a || addr >= *a + ent.size {
                Err(MemAccessError {
                    addr,
                    err: MemAccessErrorType::NotInRange(*a, ent.size),
                })
            } else {
                Ok(ent)
            }
        } else {
            Err(MemAccessError {
                addr,
                err: MemAccessErrorType::NoEntry,
            })
        }
    }
}

impl BusDevice for BTreeBus {
    fn validate(&mut self, _base_addr: u32, _size: u32) {
    }

    fn read(&mut self, addr: u32, size: u32) -> Result<u32, MemAccessError> {
        let ent = self.bus_lookup(addr)?;
        let val = ent.device.read(addr - ent.addr, size);
        match size {
            8 => val,
            16 => val.map(|x| self.endianness.read_u16(x as u16) as u32),
            32 => val.map(|x| self.endianness.read_u32(x)),
            _ => Err(MemAccessError {
                addr,
                err: MemAccessErrorType::BadSize,
            }),
        }
    }

    fn write(&mut self, addr: u32, size: u32, value: u32) -> Result<(), MemAccessError> {
        let val_e = match size {
            8 => value,
            16 => self.endianness.read_u16(value as u16) as u32,
            32 => self.endianness.read_u32(value),
            _ => {
                return Err(MemAccessError {
                    addr,
                    err: MemAccessErrorType::BadSize,
                });
            }
        };

        let ent = self.bus_lookup(addr)?;
        ent.device.write(addr - ent.addr, size, val_e)
    }
}

#[cfg(test)]
mod test {
    use crate::cpu::bus::BusDevice;
    struct SingleMemoryAddress {
        value: u32,
    }

    struct ReadOnly {
        value: u32,
    }

    impl super::BusDevice for SingleMemoryAddress {
        fn validate(&mut self, _base_addr: u32, _size: u32) {}
        fn read(&mut self, _addr: u32, _size: u32) -> Result<u32, super::MemAccessError> {
            Ok(self.value)
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
        fn read(&mut self, _addr: u32, _size: u32) -> Result<u32, super::MemAccessError> {
            Ok(self.value)
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
        let mut bus = super::BTreeBus::default();
        let mem_value = 2048;

        let dev = Box::new(SingleMemoryAddress { value: mem_value });
        bus.map(0x1000, 0x1000, dev);

        match bus.read(0x1004, 32) {
            Ok(v) => assert_eq!(mem_value, v),
            Err(e) => panic!("Memory error {:?}", e),
        }
    }

    #[test]
    fn bus_test_read_write() {
        let mut bus = super::BTreeBus::default();
        let mem_value = 2048;

        let dev = Box::new(SingleMemoryAddress { value: mem_value });
        bus.map(0x1000, 0x1000, dev);

        match bus.read(0x1004, 32) {
            Ok(v) => assert_eq!(mem_value, v),
            Err(e) => panic!("Memory error {:?}", e),
        }

        let new_value = 10;
        match bus.write(0x1004, 32, new_value) {
            Ok(_) => {}
            Err(e) => panic!("Memory error on write {:?}", e),
        }

        match bus.read(0x1004, 32) {
            Ok(v) => assert_eq!(new_value, v),
            Err(e) => panic!("Memory error {:?}", e),
        }
    }

    #[test]
    fn bus_test_read_only_write_err() {
        let mut bus = super::BTreeBus::default();
        let mem_value = 2048;

        let dev = Box::new(ReadOnly { value: mem_value });
        bus.map(0x1000, 0x1000, dev);

        match bus.read(0x1004, 32) {
            Ok(v) => assert_eq!(mem_value, v),
            Err(e) => panic!("Memory error {:?}", e),
        }

        match bus.write(0x1004, 32, 10) {
            Ok(_) => panic!("Write succeeded for RO memory region"),
            Err(_) => {}
        }

        match bus.read(0x1004, 32) {
            Ok(v) => assert_eq!(mem_value, v),
            Err(e) => panic!("Memory error {:?}", e),
        }
    }

    #[test]
    fn bus_test_unmapped_error() {
        let mut bus = super::BTreeBus::default();
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
        let mut bus = super::BTreeBus::default();
        let mem_value = 2048;

        let dev1 = Box::new(SingleMemoryAddress { value: mem_value });
        bus.map(0x1000, 0x1000, dev1);

        let dev2 = Box::new(SingleMemoryAddress { value: mem_value });
        bus.map(0x1000, 0x1000, dev2);
    }
}
