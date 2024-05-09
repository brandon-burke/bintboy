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

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

use crate::gameboy::cpu::{Cpu, cpu_state};
use crate::gameboy::memory::Memory;
use crate::TestStatus;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

pub struct Gameboy {
    cpu: Cpu,
    memory: Memory,
}

impl Gameboy {
    pub fn new() -> Self {
        Gameboy { 
            cpu: Cpu::new(), 
            memory: Memory::new()
        }
    }

    /**
     * This will load the game data's ram bank 0 and 1 into the gameboy. As well
     * if SRAM is present, then it will load sram bank 0 into the gameboy. Finally
     * this will also setup the mbc register for memory, which will house all 
     * the information of the game cartridge
     */
    pub fn initialize(&mut self, rom_file_path: &str) {
        self.memory.game_data.load_rom(rom_file_path);
        self.memory.initialize_game_data();
        self.memory.mbc_reg.bank_bit_mask = self.memory.game_data.bank_bit_mask();
        self.memory.mbc_reg.mbc_type = self.memory.game_data.cartridge_type();
        self.memory.mbc_reg.ram_size = self.memory.game_data.ram_size();
        self.memory.mbc_reg.rom_size = self.memory.game_data.rom_size();
        println!("{:?}", self.memory.mbc_reg);
    }

    /**
     * This is the starting point for the Game Boy. You just need to give it a
     * rom file for it to run
     */
    pub fn run(&mut self) {
        let mut buffer = vec![0u32; WIDTH * WIDTH];
        let mut buffer_index: usize = 0;
        let buff_max = WIDTH * HEIGHT;
        let mut window = Self::initialize_window();
        self.memory.ppu.activate_ppu();
        
        while window.is_open() && !window.is_key_down(Key::Escape) {
            self.memory.timer_cycle();
            self.memory.dma_cycle();
            self.memory.joypad_cycle(&window);
            if self.memory.ppu.is_active() {
                self.memory.gpu_cycle(&mut buffer, &mut buffer_index);
            }

            if buffer_index == buff_max {
                buffer_index = 0;
                window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
            }

            //Only try to service an interrupt if you finished an instruction
            match self.cpu.cpu_state {
                cpu_state::CpuState::Fetch => self.memory.interrupt_cycle(&mut self.cpu.pc, &mut self.cpu.sp),
                _ => (),
            }

            if !self.memory.interrupt_handler.handling_isr {
                self.cpu.cycle(&mut self.memory);
            }
        }
    }

    fn initialize_window() -> Window {
        let mut window = Window::new(
            "Noise Test - Press ESC to exit",
            WIDTH,
            HEIGHT,
            WindowOptions {
                resize: false,
                title: true,
                scale: Scale::X1,
                scale_mode: ScaleMode::Stretch,
                ..WindowOptions::default()
            },
        )
            .expect("Unable to create the window");

        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        return window;
    }
    
    /**
     * This is the starting point for the Game Boy. You just need to give it a
     * rom file for it to run
     */
    pub fn test_run(&mut self) -> TestStatus {
        let mut buffer = vec![0u32; WIDTH * HEIGHT];
        let mut buffer_index: usize = 0;
        let buff_max = WIDTH * HEIGHT;
        let mut window = Self::initialize_window();
        self.memory.ppu.activate_ppu();
        
        while window.is_open() && !window.is_key_down(Key::Escape) {
            let new_size = window.get_size();

            self.memory.timer_cycle();
            self.memory.dma_cycle();
            self.memory.joypad_cycle(&window);
            if self.memory.ppu.is_active() {
                self.memory.gpu_cycle(&mut buffer, &mut buffer_index);
            }

            if buffer_index == buff_max {
                buffer_index = 0;
                window.update_with_buffer(&buffer, new_size.0, new_size.1).unwrap();
            }
            //Only try to service an interrupt if you finished an instruction
            match self.cpu.cpu_state {
                cpu_state::CpuState::Fetch => self.memory.interrupt_cycle(&mut self.cpu.pc, &mut self.cpu.sp),
                _ => (),
            }
            if !self.memory.interrupt_handler.handling_isr {
                self.cpu.cycle(&mut self.memory);
            }

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

        return TestStatus::Pass;
    }

}

