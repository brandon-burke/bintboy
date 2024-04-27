mod cpu;
mod memory;
mod ppu;
mod timer;
mod serial_transfer;
mod joypad;
mod dma;
mod interrupt_handler;
mod opcodes;
mod binary_utils;
mod constants;

use minifb::{Key, ScaleMode, Window, WindowOptions};

use crate::gameboy::cpu::{Cpu, cpu_state};
use crate::gameboy::memory::Memory;

pub enum TestStatus {
    Pass,
    Failed,
}

pub struct Gameboy {
    cpu: Cpu,
    memory: Memory,
}

impl Gameboy {
    pub fn new() -> Self {
        Gameboy { 
            cpu: Cpu::new(), 
            memory: Memory::new(), 
        }
    }

    /**
     * This is the starting point for the gameboy. You just need to give it a 
     * rom file for it to run
     */
    pub fn run(&mut self, rom_0: [u8; 0x4000], rom_1: [u8; 0x4000], is_blargg_test: bool) -> TestStatus {
        let mut blargg_buffer = "".to_string();
        let blargg_pass_value = "Passed";
        let blargg_failed_value = "Failed";
        let mut start_caring = false;
        let mut passing_clk_ticks = 0;
        let mut stupid = false;

        //Loading rom into memory. Note we're probably going to need to add some
        //Logic to load roms with higher capacities because this just does a 32k
        self.memory.load_rom(rom_0, rom_1);
    
        const WIDTH: usize = 160;
        const HEIGHT: usize = 144;
        let mut buffer = vec![0u32; WIDTH * HEIGHT];
        let mut buffer_index: usize = 0;
        let buff_max = WIDTH * HEIGHT;
        let mut window = Window::new(
            "Noise Test - Press ESC to exit",
            WIDTH,
            HEIGHT,
            WindowOptions {
                resize: true,
                scale_mode: ScaleMode::UpperLeft,
                ..WindowOptions::default()
            },
        )
        .expect("Unable to create the window");
    
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        while window.is_open() && !window.is_key_down(Key::Escape) {
            if passing_clk_ticks > 0 {
                passing_clk_ticks -= 1;
                if passing_clk_ticks == 0 {
                    return TestStatus::Pass;
                }
            }

            let new_size = window.get_size();

            self.memory.timer_cycle();
            self.memory.dma_cycle();

            if self.memory.ppu.is_active() {
                self.memory.gpu_cycle(&mut buffer, &mut buffer_index);
            }

            if buffer_index == buff_max {
                //println!("buf max found");
                buffer_index = 0;

                window
                .update_with_buffer(&buffer, new_size.0, new_size.1)
                .unwrap();
            }

            if !self.memory.interrupt_handler.handling_isr {
                self.cpu.cycle(&mut self.memory, is_blargg_test);

                if self.cpu.current_opcode == 0x40 && !is_blargg_test {
                    if self.cpu.b == 66 && self.cpu.c == 66 && self.cpu.d == 66
                        && self.cpu.e == 66 && self.cpu.h == 66 && self.cpu.l == 66 {

                        return TestStatus::Failed;
                    }

                    if self.cpu.b == 3 && self.cpu.c == 5 && self.cpu.d == 8
                        && self.cpu.e == 13 && self.cpu.h == 21 && self.cpu.l == 34 {

                        return TestStatus::Pass;
                    }
                }
            }
    
            //Only try to service an interrupt if you finished an instruction
            match self.cpu.cpu_state {
                cpu_state::CpuState::Fetch => self.memory.interrupt_cycle(&mut self.cpu.pc, &mut self.cpu.sp),
                _ => (),
            }        
    
            if self.memory.read_byte(0xff02) == 0x81 && is_blargg_test {
                let byte = self.memory.read_byte(0xff01);
                print!("{}", byte as char);

                if (byte as char == 'P' || byte as char == 'F') && !start_caring {
                    start_caring = true;
                }

                if start_caring {
                    blargg_buffer.push(byte as char);
                }

                if blargg_buffer.len() == blargg_pass_value.len() {
                    if blargg_buffer == blargg_pass_value {
                        return TestStatus::Pass;
                    } else if blargg_buffer == blargg_failed_value {
                        return TestStatus::Failed;
                    } else {
                        panic!("ERROR: |{blargg_buffer}|");
                    }
                }

                self.memory.write_byte(0xff02, 0);
            }
        }

        return TestStatus::Pass;


        // loop {
        //     self.memory.timer_cycle();
        //     self.memory.dma_cycle();

        //     self.memory.gpu_cycle();

        //     if !self.memory.interrupt_handler.handling_isr {
        //         self.cpu.cycle(&mut self.memory, is_blargg_test);

        //         if self.cpu.current_opcode == 0x40 && !is_blargg_test {
        //             if self.cpu.b == 66 && self.cpu.c == 66 && self.cpu.d == 66 
        //                 && self.cpu.e == 66 && self.cpu.h == 66 && self.cpu.l == 66 {
        //                 return TestStatus::Failed;
        //             }

        //             if self.cpu.b == 3 && self.cpu.c == 5 && self.cpu.d == 8 
        //                 && self.cpu.e == 13 && self.cpu.h == 21 && self.cpu.l == 34 {
        //                     return TestStatus::Pass;
        //             }
        //         }
        //     }
    
        //     //Only try to service an interrupt if you finished an instruction
        //     match self.cpu.cpu_state {
        //         cpu_state::CpuState::Fetch => self.memory.interrupt_cycle(&mut self.cpu.pc, &mut self.cpu.sp),
        //         _ => (),
        //     }        
    
        //     if self.memory.read_byte(0xff02) == 0x81 && is_blargg_test{
        //         let byte = self.memory.read_byte(0xff01);
        //         //print!("{}", byte as char);

        //         if (byte as char == 'P' || byte as char == 'F') && !start_caring { 
        //             start_caring = true; 
        //         }

        //         if start_caring {
        //             blargg_buffer.push(byte as char);
        //         }

        //         if blargg_buffer.len() == blargg_pass_value.len() {
        //             if blargg_buffer == blargg_pass_value {
        //                 return TestStatus::Pass;
        //             } else if blargg_buffer == blargg_failed_value {
        //                 return TestStatus::Failed;
        //             } else {
        //                 panic!("ERROR: |{blargg_buffer}|");
        //             }
        //         }

        //         self.memory.write_byte(0xff02, 0);
        //     }
        // }
    }
}

