mod gameboy;
use crate::gameboy::Gameboy;

use std:: fs;
use std::fs::File;
use std::io::Read;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Will take in a single file path to a .gb rom test file
    F { path: String },

    /// Will take in a directory path and will run all .gb rom test files in it
    D { path: String },

    /// Will take in a single file path to a .gb rom test file. Blargg tests
    FB { path: String },

    /// Will take in a directory path and will run all .gb rom test files in it. Blargg tests
    DB { path: String },
}

/**
 * THINGS I TOLD MYSELF WOULD BE A PROBLEM LATER BUT DIDNT LISTEN
 * 
 * -If the game isn't running well. Could be due to a bunch of memory copying from popping the head of Vector types
 *      might be better to use something that doesn't have allocation penalties from popping from the head.
 * -Not letting sprites physically draw over the window because of how I mix pixels
 * -Not using the WX reg for pixel fetching, so window drawing can be wrong because of this
 * -When constructing the pixels and xpos is flipped I'm pushing to the head to yeah memory shifting
 * -Remember to remove all the unused linting (#![allow(dead_code)])
 */

/**
 * This is where the gameboy emulator starts. It takes in command line arguments
 * that specify what gameboy rom to run
 */
fn main() {
    let args = std::env::args();
    println!("{:?}", args);
    let args = Cli::parse();

    match args.cmd {
        Commands::F { path } => single_test_rom_run(&path, false),
        Commands::D { path } => multiple_test_rom_run(&path, false),
        Commands::FB { path } => single_test_rom_run(&path, true),
        Commands::DB { path } => multiple_test_rom_run(&path, true),
    }
}

fn single_test_rom_run(path: &str, is_blargg_test: bool) {
    let (rom_file_0, rom_file_1) = create_rom_file(path);
    let mut gameboy = Gameboy::new();
    println!("Running Test {}", path);
    match gameboy.run(rom_file_0, rom_file_1, is_blargg_test) {
        gameboy::TestStatus::Pass => println!("Pass"),
        gameboy::TestStatus::Failed => println!("Failed"),
    };
}

fn multiple_test_rom_run(path: &str, is_blargg_test: bool) {
    let paths = fs::read_dir(path).unwrap();
    let mut tests = vec![];
    for path in paths {
        let path = path.unwrap().path();
        if path.is_file() && path.extension().unwrap() == "gb" {
            println!("Reading {}", &path.display().to_string());
            let (rom_file_0, rom_file_1) = create_rom_file(&path.display().to_string());
            let mut gameboy = Gameboy::new();

            let result = match gameboy.run(rom_file_0, rom_file_1, is_blargg_test) {
                gameboy::TestStatus::Pass => "Pass",
                gameboy::TestStatus::Failed => "Failed",
            };

            tests.push((path.file_name().unwrap().to_str().unwrap().to_owned(), result))
        }
    }

    let mut num_of_failures = 0;
    for (test, status) in tests {
        if status == "Failed" {
            num_of_failures += 1;
        }
        println!("{}: {}", test, status);
    }

    if num_of_failures == 0 {
        println!("\n*** ALL TESTS PASSED ***")
    } else {
        if num_of_failures == 1 {
            println!("\n*** {num_of_failures} TEST FAILURE ***");
        } else {
            println!("\n*** {num_of_failures} TESTS FAILURES ***");
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
        // if i == 16384 {
        //     break;
        // }

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

    println!("MBC type |{}|", rom_file_0[0x147]);
    println!("ROM size|{}|", rom_file_0[0x148]);
    println!("RAM size|{}|", rom_file_0[0x149]);

    return (rom_file_0, rom_file_1);
}
