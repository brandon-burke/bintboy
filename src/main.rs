mod gameboy;

use std::env;
use std::fs::File;
use std::io::Read;
use crate::gameboy::Gameboy;

/**
 * This is where the gameboy emulator starts. It takes in command line arguments
 * that specify what gameboy rom to run
 */
fn main() {
    let args = env::args().collect::<Vec<String>>();
    let (rom_file_0, rom_file_1) = create_rom_file(&args[1]);
    let gameboy = Gameboy::new();

    gameboy.run(rom_file_0, rom_file_1);
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
