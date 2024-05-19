mod gameboy;
mod game_cartridge;

use std::{fs::File, io::{Read, Write}};

use crate::gameboy::Gameboy;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    path: String,
    save_state: Option<String>
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
 * -NOT IMPLEMENTING THE MBC ENTIRELY
 * -Not making mbc1m its own struct. Because right now we have to do a comparison any time you write 
 * -MBC3's timer isn't really implemented 
 */

/**
 * Main entry point which will take in a file path the user specifies
 */
fn main() {
    let args = Cli::parse();
    start_emulator(&args.path, args.save_state);
}

/* This is the entry point for the Game Boy emulator */
fn start_emulator(rom_file_path: &str, gameboy_save_state: Option<String>) {
    let mut gameboy = match gameboy_save_state {
        Some(ref save_state_path) => {
            let save_file = File::open(save_state_path).expect("Save state file not found");
            let mut deserialized_gameboy: Vec<u8> = vec![];
            for byte in save_file.bytes() {
                deserialized_gameboy.push(byte.unwrap());
            } 
            let mut gameboy: Gameboy = bincode::deserialize(&deserialized_gameboy).unwrap();

            gameboy.initialize_rom_only(rom_file_path);

            gameboy
        },
        None => {
            let mut gameboy = Gameboy::new();
            gameboy.initialize(rom_file_path);

            gameboy
        },
    };

    //Start the Game Boy
    gameboy.run();

    //Checking if the gamer wants to save the state of their game
    println!("Do you want to save the state of the rom? (y/n)");
    let mut user_input = String::new();
    let _ = std::io::stdin().read_line(&mut user_input);
    if user_input.trim() == "y" {
        //Seeing if the gamer wants to use the same save file if they provided one
        let use_existing_save_file = match gameboy_save_state {
            Some(ref save_state_path) => {
                println!("Do you want overwrite your existing save file named {} (y/n)", save_state_path);
                user_input.clear();
                let _ = std::io::stdin().read_line(&mut user_input);
                if user_input.trim() == "y" {
                    true
                } else {
                    false
                }
            },
            None => false,
        };


        let save_file_path = if use_existing_save_file {
            gameboy_save_state.unwrap()
        } else {
            println!("Type the file name you want to create or overwrite");
            user_input.clear();
            let _ = std::io::stdin().read_line(&mut user_input);

            user_input
        };

        //let serialized_gameboy = serde_json::to_string(&gameboy).unwrap();
        let serialized_gameboy = bincode::serialize(&gameboy).unwrap();
        let mut file = File::create(save_file_path.trim()).unwrap();
        let _ = file.write(&serialized_gameboy);
    }
}

/* This is the entry point for the Game Boy emulator */
#[allow(unused)]
fn test_start_emulator(rom_file_path: &str) -> TestStatus {
    let mut gameboy = Gameboy::new();
    gameboy.initialize(rom_file_path);
    gameboy.test_run()
}

enum TestStatus {
    Failed,
    Pass
}

#[cfg(test)]
mod tests {
    use std::fs;
    use colored::Colorize;

    use crate::{test_start_emulator, TestStatus};

    /*
        This will run all the blargg test ROMs individually, which are each 32KB in size. This test
        is helpful when your Game Boy emulator can only support ROMs up to 32KB.
    */
    // #[test]
    // fn run_individual_blargg_roms() {
    //     let path = "test_roms/individual/";
    //     let paths = fs::read_dir(path).unwrap();
    //     //let mut tests = vec![];
    //     for path in paths {
    //         let path = path.unwrap().path();
    //         if path.is_file() && path.extension().unwrap() == "gb" {
    //             println!("Reading {}", &path.display().to_string());
    //             start_emulator(&path.display().to_string());
    //             //tests.push((path.file_name().unwrap().to_str().unwrap().to_owned(), result))
    //         }
    //     }
    
    //     // let mut num_of_failures = 0;
    //     // for (test, status) in tests {
    //     //     if status == "Failed" {
    //     //         num_of_failures += 1;
    //     //     }
    //     //     println!("{}: {}", test, status);
    //     // }
    
    //     // if num_of_failures == 0 {
    //     //     println!("\n*** ALL TESTS PASSED ***")
    //     // } else {
    //     //     if num_of_failures == 1 {
    //     //         println!("\n*** {num_of_failures} TEST FAILURE ***");
    //     //     } else {
    //     //         println!("\n*** {num_of_failures} TESTS FAILURES ***");
    //     //     }
    //     // }
        
    // }

    #[test]
    fn run_individual_mooneye_roms() {
        let test_roms_path_list = vec![("test_roms/acceptance/bits", "BITS TEST"), 
                            ("test_roms/acceptance/instr", "INSTR TEST"), 
                            ("test_roms/acceptance/oam_dma", "OAM_DMA TEST"), 
                            ("test_roms/acceptance/timer", "TIMER TEST"), 
                            ("test_roms/acceptance/interrupts", "INTERRUPT TEST"),
                            ("test_roms/emulator-only/mbc1", "MBC1 TEST"),
                            ("test_roms/emulator-only/mbc5", "MBC5 TEST"),
                            ];
        
        let mut num_of_failures = 0;
        for (test_rom_folder_path, test_name) in test_roms_path_list {
            //Printing out the Test Section Name
            let msg = format!("\n{}", test_name);
            println!("{}", msg.bright_cyan());
            println!("===============================");

            //Going through all the test roms and printing out the results once
            //the rom finishes
            let test_rom_folder = fs::read_dir(test_rom_folder_path).unwrap();
            for rom_path in test_rom_folder {
                let rom = rom_path.unwrap().path();
                if rom.is_file() && rom.extension().unwrap() == "gb" {
                    match test_start_emulator(&rom.display().to_string()) {
                        TestStatus::Failed => {
                            println!("{}: {}", rom.file_name().unwrap().to_str().unwrap(), "Failed".red());
                            num_of_failures += 1;
                        },
                        TestStatus::Pass => println!("{}: {}", rom.file_name().unwrap().to_str().unwrap(), "Pass".green()),
                    };
                }
            }
        }
    
        if num_of_failures == 0 {
            let msg = String::from("\n*** ALL TESTS PASSED ***\n\n");
            println!("{}", msg.green());
            assert!(true);
        } else {
            if num_of_failures == 1 {
                let msg = format!("\n*** {num_of_failures} TEST FAILURE ***\n\n");
                println!("{}", msg.red());
            } else {
                let msg = format!("\n*** {num_of_failures} TESTS FAILURES ***\n\n");
                println!("{}", msg.red());
            }
            assert!(false);
        }
    }
}