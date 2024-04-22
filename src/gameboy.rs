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
    pub fn run(&mut self, rom_0: [u8; 0x4000], rom_1: [u8; 0x4000]) -> TestStatus {
        //Loading rom into memory. Note we're probably going to need to add some
        //Logic to load roms with higher capacities because this just does a 32k
        self.memory.load_rom(rom_0, rom_1);

        loop {
            self.memory.timer_cycle();
            self.memory.dma_cycle();
            self.memory.gpu_cycle();
            if !self.memory.interrupt_handler.handling_isr {
                self.cpu.cycle(&mut self.memory);

                if self.cpu.current_opcode == 0x40 {
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
    
            // if self.memory.read_byte(0xff02) == 0x81 {
            //     let byte = self.memory.read_byte(0xff01);
            //     print!("{}", byte as char);
            //     self.memory.write_byte(0xff02, 0);
            // }
            

        }
    }
}

