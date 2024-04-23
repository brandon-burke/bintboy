mod gameboy;

use std::{env, fs};
use std::fs::File;
use std::io::Read;
use crate::gameboy::Gameboy;

/**
 * THINGS I TOLD MYSELF WOULD BE A PROBLEM LATER BUT DIDNT LISTEN
 * 
 * -If the game isn't running well. Could be due to a bunch of memory copying from popping the head of Vector types
 *      might be better to use something that doesn't have allocation penalties from popping from the head.
 * -Not letting sprites physically draw over the window because of how I mix pixels
 * -Not using the WX reg for pixel fetching, so window drawing can be wrong because of this
 * -When constructing the pixels and xpos is flipped I'm pushing to the head to yeah memory shifting
 */

/**
 * This is where the gameboy emulator starts. It takes in command line arguments
 * that specify what gameboy rom to run
 */
fn main() {
    let args = env::args().collect::<Vec<String>>();
    //let file_name = &args[1];
    let file_name = "test_roms/acceptance/bits/mem_oam.gb";
    let debug = false;

    if !debug && &args[2] == "1" {
        let paths = fs::read_dir(&args[1]).unwrap();
        let mut tests = vec![];
        for path in paths {
            let path = path.unwrap().path();
            if path.is_file() && path.extension().unwrap() == "gb" {
                println!("Reading {}", &path.display().to_string());
                let (rom_file_0, rom_file_1) = create_rom_file(&path.display().to_string());
                let mut gameboy = Gameboy::new();
    
                let result = match gameboy.run(rom_file_0, rom_file_1) {
                    gameboy::TestStatus::Pass => "Pass",
                    gameboy::TestStatus::Failed => "Failed",
                };
    
                tests.push((path.file_name().unwrap().to_str().unwrap().to_owned(), result))
            }
        }

        for (test, status) in tests {
            println!("{}: {}", test, status);
        }
    } else {
        let (rom_file_0, rom_file_1) = create_rom_file(file_name);
        let mut gameboy = Gameboy::new();
        println!("Reading {}", file_name);
        match gameboy.run(rom_file_0, rom_file_1) {
            gameboy::TestStatus::Pass => println!("Pass"),
            gameboy::TestStatus::Failed => println!("Failed"),
        };
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
