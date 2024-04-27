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
use crate::rom::Rom;

pub struct Gameboy {
    cpu: Cpu,
    memory: Memory,
    rom: Rom,
}

impl Gameboy {
    pub fn new() -> Self {
        Gameboy { 
            cpu: Cpu::new(), 
            memory: Memory::new(),
            rom: Rom::new(),
        }
    }

    /**
     * This will load the game data from a file into the gameboy. As well
     * load the ROM memory region(0x0000 - 0x7FFF) on the game boy with the 
     * first 16KB banks of the game data.
     */
    pub fn load_rom(&mut self, rom_file_path: &str) {
        self.rom.load_rom(rom_file_path);
        self.memory.copy_game_data_to_rom(self.rom.banks[0], self.rom.banks[1]);
    }

    /**
     * This is the starting point for the gameboy. You just need to give it a 
     * rom file for it to run
     */
    pub fn run(&mut self) {
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
            let new_size = window.get_size();

            self.memory.timer_cycle();
            self.memory.dma_cycle();
            if self.memory.ppu.is_active() {
                self.memory.gpu_cycle(&mut buffer, &mut buffer_index);
            }

            if buffer_index == buff_max {
                buffer_index = 0;
                window.update_with_buffer(&buffer, new_size.0, new_size.1).unwrap();
            }

            if !self.memory.interrupt_handler.handling_isr {
                self.cpu.cycle(&mut self.memory);
            }
    
            //Only try to service an interrupt if you finished an instruction
            match self.cpu.cpu_state {
                cpu_state::CpuState::Fetch => self.memory.interrupt_cycle(&mut self.cpu.pc, &mut self.cpu.sp),
                _ => (),
            }     
        }
    }
}

