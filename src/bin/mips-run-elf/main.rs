use libpsx::cpu::bus::{BusDevice, SizedReadResult};
use object::{Object, ObjectSection};

fn load_section(bus: &mut dyn BusDevice, addr: u32, buf: &[u8]) {
    let len = buf.len();

    for i in 0..len {
        bus.write(addr + i as u32, 8, buf[i] as u32).unwrap();
    }
}

struct DiscountUart;

impl libpsx::cpu::bus::BusDevice for DiscountUart {
    fn validate(&mut self, _base_addr: u32, _size: u32) {}

    fn write(
        &mut self,
        _addr: u32,
        _size: u32,
        value: u32,
    ) -> Result<(), libpsx::cpu::bus::MemAccessError> {
        print!("{}", (value as u8) as char);
        Ok(())
    }

    fn read(&mut self, addr: u32, _size: u32) -> Result<SizedReadResult, libpsx::cpu::bus::MemAccessError> {
        Err(libpsx::cpu::bus::MemAccessError {
            addr,
            err: libpsx::cpu::bus::MemAccessErrorType::ReadOnly,
        })
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let buf: Vec<u8> = std::fs::read(&args[1]).unwrap();
    let obj = object::File::parse(&*buf).unwrap();

    if obj.architecture() != object::Architecture::Mips {
        panic!("Not a MIPS ELF file");
    }

    let ram = Box::new(libpsx::mem::memory::RAM::new(1 << 25));
    let uart = Box::new(DiscountUart);

    let mut bus = libpsx::cpu::bus_vec::VecBus::default();
    bus.endianness = obj.endianness();
    bus.map(0x0, 1 << 25, ram);
    bus.map(0x1fd003f8, 0x10, uart);

    for section in obj.sections() {
        load_section(
            &mut bus,
            section.address().try_into().unwrap(),
            section.data().unwrap(),
        );
    }

    let mut state = libpsx::cpu::jit::CpuState::default();
    state.set_pc(obj.entry() as u32);
    libpsx::cpu::jit::execute(&mut bus, &mut state).unwrap();
}
