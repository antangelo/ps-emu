use std::io::Seek;
use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    //let file = match elf::File::open_path(&args[0]) {
    //    Ok(f) => f,
    //    Err(e) => panic!("Error: {:?}", e),
    //};

    //let text = match file.get_section(".text") {
    //    Some(s) => s,
    //    None => panic!("No text section"),
    //};

    //let mut instrs: Vec<u32> = vec![0;text.data.len()/4];

    //assert!(text.data.len() % 4 == 0);

    let buf: Vec<u8> = std::fs::read(&args[1]).unwrap();
    let flen = buf.len() / 4;

    println!("{}", flen);
    for i in 0..flen {
        let instr = u32::from_be_bytes(buf[4*i..4*i+4].try_into().unwrap());
        println!("{:#08x} \t {}", instr, libpsx::cpu::mips_decode(instr));
    }
}

