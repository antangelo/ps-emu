use crate::cpu::{bus, bus::MemAccessError, bus::MemAccessErrorType};
use object::Endian;

pub struct RAM {
    endianness: object::Endianness,
    mem: Box<[u8]>,
    size: u32,
}

impl RAM {
    pub fn new(size: u32) -> Self {
        // Endianness is managed on the bus level, but it is convenient
        // to use it to read bytes into u32/u16s as well
        // so we use the system endianess to do it.
        let endianness = if object::NativeEndian.is_little_endian() {
            object::Endianness::Little
        } else {
            object::Endianness::Big
        };

        RAM {
            endianness,
            mem: vec![0; size as usize].into_boxed_slice(),
            size,
        }
    }
}

impl bus::BusDevice for RAM {
    fn validate(&mut self, _base_addr: u32, size: u32) {
        assert!(self.size >= size);
    }

    fn read(&mut self, addr: u32, size: u32) -> Result<u32, MemAccessError> {
        unsafe {
            let mem: *const u8 = self.mem.as_ptr().add(addr as usize);
            match size {
                8 => Ok(*mem as u32),
                16 => Ok(self.endianness.read_u16(*(mem as *const u16)) as u32),
                32 => Ok(self.endianness.read_u32(*(mem as *const u32))),
                _ => Err(MemAccessError {
                    addr,
                    err: MemAccessErrorType::BadSize,
                }),
            }
        }
    }

    fn write(&mut self, addr: u32, size: u32, value: u32) -> Result<(), MemAccessError> {
        unsafe {
            let mem: *mut u8 = self.mem.as_mut_ptr().add(addr as usize);
            match size {
                8 => {
                    *mem = value as u8;
                },
                16 => {
                    *(mem as *mut u16) = self.endianness.write_u16(value as u16);
                },
                32 => {
                    *(mem as *mut u32) = self.endianness.write_u32(value);
                },
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
    #[test]
    fn ram_test_generic() {
        let ram = Box::new(super::RAM::default());
        let mut bus = super::bus::Bus::default();

        bus.map(0x0, 0x1000, ram);

        match bus.read(0x0, 32) {
            Ok(v) => assert_eq!(0, v),
            Err(e) => panic!("Memory error {:?}", e),
        }

        let value = 1024;
        match bus.write(0x0, 32, value) {
            Ok(_) => {}
            Err(e) => panic!("Memory write error {:?}", e),
        }

        match bus.read(0x0, 32) {
            Ok(v) => assert_eq!(value, v),
            Err(e) => panic!("Memory error {:?}", e),
        }

        match bus.read(0x0, 8) {
            Ok(v) => assert_eq!((value & 0xff) as u8, v as u8),
            Err(e) => panic!("Memory error {:?}", e),
        }
    }
}
