use object::{Object, ObjectSection, Endian};

fn mips_disassemble_bus(bus: &mut libpsx::cpu::bus::Bus, addr: u32, size: usize) {
    for i in 0..size {
        let instr = bus.read(addr + 4 * (i as u32), 32).unwrap();
        println!("{:#08x} \t {:#08x} \t {}", addr + 4 * (i as u32), instr, libpsx::cpu::decode::mips_decode(instr));
    }
}

fn mips_load_text(bus: &mut libpsx::cpu::bus::Bus, addr: u32, buf: &[u8]) {
    let len = buf.len();

    for i in 0..len {
        bus.write(addr + i as u32, 8, buf[i] as u32).unwrap();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let buf: Vec<u8> = std::fs::read(&args[1]).unwrap();
    let obj = object::File::parse(&*buf).unwrap();

    if obj.architecture() != object::Architecture::Mips {
        panic!("Not a MIPS ELF file");
    }

    let ram = Box::new(libpsx::mem::memory::RAM::default());

    let mut bus = libpsx::cpu::bus::Bus::default();
    bus.endianness = obj.endianness();
    bus.map(0x0, 1 << 25, ram); 

    let text_addr: u32;
    let text_size: u32;

    if let Some(text) = obj.section_by_name(".text") {
        text_addr = text.address().try_into().unwrap();
        text_size = text.size().try_into().unwrap();
        mips_load_text(&mut bus, text_addr, text.data().unwrap()); 
    } else {
        panic!("No .text section found");
    }

    mips_disassemble_bus(&mut bus, text_addr, text_size as usize);
}

