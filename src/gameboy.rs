mod cpu;
mod memory;

use crate::gameboy::cpu::Cpu;
use crate::gameboy::memory::Memory;

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
    pub fn run(&self, rom_0: [u8; 0x4000], rom_1: [u8; 0x4000]) {
        //Loading rom into memory. Note we're probably going to need to add some
        //Logic to load roms with higher capacities because this just does a 32k
        self.memory.load_rom(rom_0, rom_1);

        loop {
            self.memory.timer_cycle();
            if !self.memory.interrupt_handler.handling_isr {
                self.cpu.cycle(&mut self.memory);
            }
    
            //Only try to service an interrupt if you finished an instruction
            match self.cpu.cpu_state {
                cpu_state::CpuState::Fetch => self.memory.interrupt_cycle(&mut self.cpu.pc, &mut self.cpu.sp),
                _ => (),
            }        
    
            if self.memory.read_byte(0xff02) == 0x81 {
                let byte = self.memory.read_byte(0xff01);
                print!("{}", byte as char);
                self.memory.write_byte(0xff02, 0);
            }
        }
    }
}

