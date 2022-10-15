use crate::cpu::{bus, bus::MemAccessError, bus::MemAccessErrorType, bus::SizedReadResult};

pub struct RAM {
    mem: Box<[u8]>,
    size: u32,
}

impl RAM {
    pub fn new(size: u32) -> Self {
        RAM {
            mem: vec![0; size as usize].into_boxed_slice(),
            size,
        }
    }
}

impl bus::BusDevice for RAM {
    fn validate(&mut self, _base_addr: u32, size: u32) {
        assert!(self.size >= size);
    }

    fn read(&mut self, addr: u32, size: u32) -> Result<SizedReadResult, MemAccessError> {
        assert!(addr < self.size);
        unsafe {
            let mem: *const u8 = self.mem.as_ptr().add(addr as usize);
            match size {
                8 => Ok(SizedReadResult::Byte(*mem)),
                16 => Ok(SizedReadResult::Word(*(mem as *const u16))),
                32 => Ok(SizedReadResult::Dword(*(mem as *const u32))),
                _ => Err(MemAccessError {
                    addr,
                    err: MemAccessErrorType::BadSize,
                }),
            }
        }
    }

    fn write(&mut self, addr: u32, size: u32, value: u32) -> Result<(), MemAccessError> {
        assert!(addr < self.size);
        unsafe {
            let mem: *mut u8 = self.mem.as_mut_ptr().add(addr as usize);
            match size {
                8 => {
                    *mem = value as u8;
                }
                16 => {
                    *(mem as *mut u16) = value as u16;
                }
                32 => {
                    *(mem as *mut u32) = value;
                }
                _ => {
                    return Err(MemAccessError {
                        addr,
                        err: MemAccessErrorType::BadSize,
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::cpu::bus::SizedReadResult;

    use super::bus::BusDevice;

    #[test]
    fn ram_test_generic() {
        let ram = Box::new(super::RAM::new(1 << 25));
        let mut bus = crate::cpu::bus_vec::VecBus::default();

        bus.map(0x0, 0x1000, ram);

        match bus.read(0x0, 32) {
            Ok(v) => assert_eq!(SizedReadResult::Dword(0), v),
            Err(e) => panic!("Memory error {:?}", e),
        }

        let value = 1024;
        match bus.write(0x0, 32, value) {
            Ok(_) => {}
            Err(e) => panic!("Memory write error {:?}", e),
        }

        match bus.read(0x0, 32) {
            Ok(v) => assert_eq!(SizedReadResult::Dword(value), v),
            Err(e) => panic!("Memory error {:?}", e),
        }

        match bus.read(0x0, 8) {
            Ok(v) => assert_eq!(SizedReadResult::Byte((value & 0xff) as u8), v),
            Err(e) => panic!("Memory error {:?}", e),
        }
    }
}
