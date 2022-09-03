use crate::cpu::{bus, bus::MemAccessError, bus::MemAccessErrorType};
use object::Endian;

pub struct RAM {
    endianness: object::Endianness,
    mem: std::vec::Vec<u8>,
}

impl RAM {
    fn read16(&self, addr: u32) -> Result<u16, MemAccessError> {
        let us_addr = addr as usize;
        Ok(self.endianness.read_u16_bytes(self.mem[us_addr..us_addr+2].try_into().unwrap()))
    }

    fn read32(&self, addr: u32) -> Result<u32, MemAccessError> {
        let us_addr = addr as usize;
        Ok(self.endianness.read_u32_bytes(self.mem[us_addr..us_addr+4].try_into().unwrap()))
    }
}

impl Default for RAM {
    fn default() -> Self {
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
            mem: vec![0; 1 << 25],
        }
    }
}

impl bus::BusDevice for RAM {
    fn read(&mut self, addr: u32, size: u32) -> Result<u32, MemAccessError> {
        match size {
            //8 => Ok(self.ram.get(&addr).map(|x| *x as u32).unwrap_or(0)),
            8 => Ok(unsafe { *self.mem.get_unchecked(addr as usize) as u32 }),
            16 => self.read16(addr).map(|x| x as u32),
            32 => self.read32(addr),
            _ => Err(MemAccessError {
                addr,
                err: MemAccessErrorType::BadSize,
            }),
        }
    }

    fn write(&mut self, addr: u32, size: u32, value: u32) -> Result<(), MemAccessError> {
        let vals = match size {
            8 => vec![value as u8],
            16 => self.endianness.write_u16_bytes(value as u16).to_vec(),
            32 => self.endianness.write_u32_bytes(value).to_vec(),
            _ => {
                return Err(MemAccessError {
                    addr,
                    err: MemAccessErrorType::BadSize,
                });
            }
        };

        for i in 0..vals.len() {
            //self.ram.insert(addr + i as u32, vals[i]);
            self.mem[(addr as usize) + i] = vals[i];
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
