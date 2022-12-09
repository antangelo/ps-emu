use std::str::FromStr;

use libpsx::cpu::bus::{BusDevice, SizedReadResult};
use object::{Object, ObjectSection};

use argparse::{ArgumentParser, Store, StoreTrue};

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

    fn read(
        &mut self,
        addr: u32,
        _size: u32,
    ) -> Result<SizedReadResult, libpsx::cpu::bus::MemAccessError> {
        Err(libpsx::cpu::bus::MemAccessError {
            addr,
            err: libpsx::cpu::bus::MemAccessErrorType::ReadOnly,
        })
    }
}

enum ExecType {
    JIT,
    Interpreter,
    ThreadedInt,
}

impl FromStr for ExecType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "jit" => Ok(Self::JIT),
            "int" | "interpreter" => Ok(Self::Interpreter),
            "thr" | "threaded" | "cached" => Ok(Self::ThreadedInt),
            _ => Err(String::from("Invalid execution mode")),
        }
    }
}

fn main() {
    let mut exec_mode = ExecType::JIT;
    let mut file = String::new();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Run a MIPS elf file");
        ap.refer(&mut exec_mode)
            .add_option(&["-m", "--mode"], Store, "Execution mode");
        ap.refer(&mut file)
            .add_argument("Object File", Store, "MIPS File")
            .required();
        ap.parse_args_or_exit();
    }

    let buf: Vec<u8> = std::fs::read(&file).unwrap();
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

    match exec_mode {
        ExecType::JIT => libpsx::cpu::jit::execute(&mut bus, &mut state),
        ExecType::Interpreter => libpsx::cpu::interpret::execute(&mut bus, &mut state),
        ExecType::ThreadedInt => libpsx::cpu::threaded::execute(&mut bus, &mut state),
    }
    .unwrap();
}
