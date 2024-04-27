mod gameboy;
mod rom;

use crate::gameboy::Gameboy;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    path: String,
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
 * Main entry point which will take in a file path the user specifies
 */
fn main() {
    let args = Cli::parse();
    start_emulator(&args.path);
}

/* This is the entry point for the Game Boy emulator */
fn start_emulator(rom_file_path: &str) {
    let mut gameboy = Gameboy::new();
    gameboy.load_rom(rom_file_path);
    gameboy.run();
}


#[cfg(test)]
mod tests {
    use std::fs;
    use crate::start_emulator;

    /*
        This will run all the blargg test ROMs individually, which are each 32KB in size. This test
        is helpful when your Game Boy emulator can only support ROMs up to 32KB.
    */
    #[test]
    fn run_individual_blargg_roms() {
        let path = "test_roms/individual/";
        let paths = fs::read_dir(path).unwrap();
        //let mut tests = vec![];
        for path in paths {
            let path = path.unwrap().path();
            if path.is_file() && path.extension().unwrap() == "gb" {
                println!("Reading {}", &path.display().to_string());
                start_emulator(&path.display().to_string());
                //tests.push((path.file_name().unwrap().to_str().unwrap().to_owned(), result))
            }
        }
    
        // let mut num_of_failures = 0;
        // for (test, status) in tests {
        //     if status == "Failed" {
        //         num_of_failures += 1;
        //     }
        //     println!("{}: {}", test, status);
        // }
    
        // if num_of_failures == 0 {
        //     println!("\n*** ALL TESTS PASSED ***")
        // } else {
        //     if num_of_failures == 1 {
        //         println!("\n*** {num_of_failures} TEST FAILURE ***");
        //     } else {
        //         println!("\n*** {num_of_failures} TESTS FAILURES ***");
        //     }
        // }
        
    }

    #[test]
    fn run_individual_mooneye_roms() {
        let path = "test_roms/acceptance/bits";
        let paths = fs::read_dir(path).unwrap();
        //let mut tests = vec![];
        for path in paths {
            let path = path.unwrap().path();
            if path.is_file() && path.extension().unwrap() == "gb" {
                println!("Reading {}", &path.display().to_string());
                start_emulator(&path.display().to_string());
                //tests.push((path.file_name().unwrap().to_str().unwrap().to_owned(), result))
            }
        }
    
        // let mut num_of_failures = 0;
        // for (test, status) in tests {
        //     if status == "Failed" {
        //         num_of_failures += 1;
        //     }
        //     println!("{}: {}", test, status);
        // }
    
        // if num_of_failures == 0 {
        //     println!("\n*** ALL TESTS PASSED ***")
        // } else {
        //     if num_of_failures == 1 {
        //         println!("\n*** {num_of_failures} TEST FAILURE ***");
        //     } else {
        //         println!("\n*** {num_of_failures} TESTS FAILURES ***");
        //     }
        // }
        
    }
}

