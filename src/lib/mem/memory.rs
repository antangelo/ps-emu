use object::Endian;
use crate::cpu::{bus, bus::MemAccessError, bus::MemAccessErrorType};

pub struct RAM {
    endianness: object::Endianness,
    ram: std::collections::BTreeMap<u32, u8>,
}

impl RAM {
    fn read16(&self, addr: u32) -> Result<u16, MemAccessError> {
        let range_it = self.ram.range(addr..addr+2);
        let mut arr: Vec<u8> = vec![0; 2];
        range_it.for_each(|x| arr[(x.0 - addr) as usize] = *(x.1));

        Ok(self.endianness.read_u16_bytes(arr.try_into().unwrap()))
    }

    fn read32(&self, addr: u32) -> Result<u32, MemAccessError> {
        let range_it = self.ram.range(addr..addr+4);
        let mut arr: Vec<u8> = vec![0; 4];
        range_it.for_each(|x| arr[(x.0 - addr) as usize] = *(x.1));

        Ok(self.endianness.read_u32_bytes(arr.try_into().unwrap()))
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

        RAM{endianness, ram: std::collections::BTreeMap::default()}
    }
}

impl bus::BusDevice for RAM {
    fn read(&mut self, addr: u32, size: u32) -> Result<u32, MemAccessError> {
        match size {
            8 => Ok(self.ram.get(&addr).map(|x| *x as u32).unwrap_or(0)),
            16 => self.read16(addr).map(|x| x as u32),
            32 => self.read32(addr),
            _ => Err(MemAccessError{addr, err: MemAccessErrorType::BadSize}),
        }
    }

    fn write(&mut self, addr: u32, size: u32, value: u32) -> Result<(), MemAccessError> {
        let vals = match size {
            8 => vec![value as u8],
            16 => self.endianness.write_u16_bytes(value as u16).to_vec(),
            32 => self.endianness.write_u32_bytes(value).to_vec(),
            _ => { return Err(MemAccessError{addr, err: MemAccessErrorType::BadSize}); }
        };

        for i in 0..vals.len() {
            self.ram.insert(addr + i as u32, vals[i]); 
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
            Ok(_) => {},
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
