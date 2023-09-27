pub mod cpu;
pub mod cpu_state;
pub mod memory;
pub mod timer;
pub mod opcodes;
pub mod binary_utils;
pub mod interrupt_handler;

//use std::env;
use std::fs::File;
use std::io::Read;
fn main() {
    //let args = env::args().collect::<Vec<String>>();
    let args = "test_roms/individual/10-bit-ops.gb";
    let (rom_file_0, rom_file_1) = create_rom_file(args);
    let mut cpu = cpu::Cpu::new();
    let mut memory = memory::Memory::new();

    memory.load_rom(rom_file_0, rom_file_1);

    loop {
        memory.timer_cycle();
        cpu.cycle(&mut memory);
        memory.interrupt_cycle();
    }
}

/**
 * Create a byte array from the ROM file
 */
fn create_rom_file(file_path: &str) -> ([u8; 0x4000], [u8; 0x4000]) {
    let file = File::open(file_path).expect("File not found");
    let mut rom_file_0 = [0; 0x4000];
    let mut rom_file_1 = [0; 0x4000];

    for (i, byte) in file.bytes().enumerate() {
        if i < 0x4000 {
            rom_file_0[i] = match byte {
                Ok(value) => value,
                Err(e) => panic!("Error: {}", e),
            };
        } else {
            rom_file_1[i - 0x4000] = match byte {
                Ok(value) => value,
                Err(e) => panic!("Error: {}", e),
            };
        }
    }

    return (rom_file_0, rom_file_1);
}
