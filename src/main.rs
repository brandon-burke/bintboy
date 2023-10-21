pub mod cpu;
pub mod cpu_state;
pub mod memory;
pub mod timer;
pub mod opcodes;
pub mod binary_utils;
pub mod interrupt_handler;
pub mod ppu;

use std::env;
use std::fs::File;
use std::io::Read;
fn main() {
    let args = env::args().collect::<Vec<String>>();
    let (rom_file_0, rom_file_1) = create_rom_file(&args[1]);
    // let args = "test_roms/individual/02-interrupts.gb";
    // let (rom_file_0, rom_file_1) = create_rom_file(args);
    let mut cpu = cpu::Cpu::new();
    let mut memory = memory::Memory::new();


    println!("Loading rom...");
    memory.load_rom(rom_file_0, rom_file_1);

    loop {
        memory.timer_cycle();
        if !memory.interrupt_handler.handling_isr {
            cpu.cycle(&mut memory);
        }
        match cpu.cpu_state {
            cpu_state::CpuState::Fetch => memory.interrupt_cycle(&mut cpu.pc, &mut cpu.sp),
            _ => (),
        }        

        if memory.read_byte(0xff02) == 0x81 {
            let byte = memory.read_byte(0xff01);
            print!("{}", byte as char);
            memory.write_byte(0xff02, 0);
        }
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
