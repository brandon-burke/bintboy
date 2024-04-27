use crate::gameboy::timer::Timer;
use crate::gameboy::joypad::Joypad;
use crate::gameboy::serial_transfer::SerialTransfer;
use crate::gameboy::dma::Dma;
use crate::gameboy::ppu::{ Ppu, enums::PpuMode };
use crate::gameboy::interrupt_handler::InterruptHandler;
use crate::gameboy::constants::*;

pub struct Memory {
    rom_bank_0: [u8; 0x4000],   //16KB -> 0000h – 3FFFh (Non-switchable ROM bank)
    rom_bank_x: [u8; 0x4000],   //16KB -> 4000h – 7FFFh (Switchable ROM bank)
    sram: [u8; 0x2000],         //8KB  -> A000h – BFFFh (External RAM in cartridge)
    wram_0: [u8; 0x1000],       //1KB  -> C000h – CFFFh (Work RAM)
    wram_x: [u8; 0x1000],       //1KB  -> D000h – DFFFh (Work RAM)
    _echo: [u8; 0x1E00],         //     -> E000h – FDFFh (ECHO RAM) Mirror of C000h-DDFFh
    unused: [u8; 0x60],         //     -> FEA0h – FEFFh (Unused)
    joypad: Joypad,             //     -> FF00h         (Joypad)
    serial: SerialTransfer,     //     -> FF01h - FF02h (Serial Transfer)
    timer: Timer,               //     -> FF04h - FF07h
    pub ppu: Ppu,                   //Pixel Processing Unit. Houses most of the graphics related memory
    dma: Dma,                   //     -> FF46h OAM DMA source address register
    io: [u8; 0x80],             //     -> FF00h – FF7Fh (I/O ports)
    pub interrupt_handler: InterruptHandler, //Will contain IE, IF, and IME registers (0xFFFF, 0xFF0F)
    hram: [u8; 0x7F],           //     -> FF80h – FFFEh (HRAM)
    ram_enable_reg: bool,
    ram_bank_num_reg: u8,
    rom_bank_num_reg: u8,
    banking_mode_sel_reg: u8,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            rom_bank_0: [0; 0x4000],   
            rom_bank_x: [0; 0x4000],         
            sram: [0; 0x2000],        
            wram_0: [0; 0x1000],       
            wram_x: [0; 0x1000],   
            _echo: [0; 0x1E00],                 
            unused: [0; 0x60],
            joypad: Joypad::new(),
            serial: SerialTransfer::new(),
            timer: Timer::new(),
            ppu: Ppu::new(),
            io: [0; 0x80],
            interrupt_handler: InterruptHandler::new(),
            dma: Dma::new(),
            hram: [0; 0x7F],
            ram_enable_reg: false,
            ram_bank_num_reg: 0,
            rom_bank_num_reg: 0,
            banking_mode_sel_reg: 0,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        //Can't read anything below OAM while DMA is going
        if self.dma.currently_transferring && address < OAM_START {
            return 0xFF;
        }

        match address {
            ROM_BANK_0_START ..= ROM_BANK_0_END => self.rom_bank_0[address as usize],
            ROM_BANK_X_START ..= ROM_BANK_X_END => self.rom_bank_x[(address - ROM_BANK_X_START) as usize],
            VRAM_START ..= VRAM_END => {
                if self.ppu.current_mode() != PpuMode::DrawingPixels || !self.ppu.is_active() {
                    match address {
                        TILE_DATA_0_START ..= TILE_DATA_0_END => self.ppu.read_tile_data_0(address),
                        TILE_DATA_1_START ..= TILE_DATA_1_END => self.ppu.read_tile_data_1(address),
                        TILE_DATA_2_START ..= TILE_DATA_2_END => self.ppu.read_tile_data_2(address),
                        TILE_MAP_0_START ..= TILE_MAP_0_END => self.ppu.read_tile_map_0(address),
                        TILE_MAP_1_START ..= TILE_MAP_1_END => self.ppu.read_tile_map_1(address),
                        _ => panic!("MEMORY READ ERROR: Should have never gotten here since we took care of all the VRAM addresses"),
                    }
                } else {
                    return 0xFF;
                }
            },
            SRAM_START ..= SRAM_END => self.sram[(address - SRAM_START) as usize],
            WRAM_0_START ..= WRAM_0_END => self.wram_0[(address - WRAM_0_START) as usize],
            WRAM_X_START ..= WRAM_X_END => self.wram_x[(address - WRAM_X_START) as usize],
            ECHO_START ..= ECHO_END => {
                let wram_address = address - 0x2000;
                match wram_address { 
                    WRAM_0_START ..= WRAM_0_END => self.wram_0[(wram_address - WRAM_0_START) as usize],
                    WRAM_X_START ..= WRAM_X_END => self.wram_x[(wram_address - WRAM_X_START) as usize],
                    _ => panic!("Issues calculating echo ram")
                }
            }
            OAM_START ..= OAM_END => {
                if (self.ppu.current_mode() != PpuMode::OamScan && self.ppu.current_mode() != PpuMode::DrawingPixels) || !self.ppu.is_active() {
                    self.ppu.read_oam(address)
                } else {
                    return 0xFF;
                }
            },
            UNUSED_START ..= UNUSED_END => {
                return self.unused[(address - UNUSED_START) as usize]
            },
            IO_START ..= IO_END => {
                match address {
                    JOYPAD_P1_REG => self.joypad.read_joypad_reg(),
                    SERIAL_SB_REG => self.serial.read_sb_reg(),
                    SERIAL_SC_REG => self.serial.read_sc_reg(),
                    TIMER_DIV_REG => self.timer.read_div(),
                    TIMER_TIMA_REG => self.timer.read_tima(),
                    TIMER_TMA_REG => self.timer.read_tma(),
                    TIMER_TAC_REG => self.timer.read_tac(),
                    LCDC_REG => self.ppu.read_lcdc_reg(),
                    STAT_REG => self.ppu.read_stat_reg(),
                    SCY_REG => self.ppu.read_scy_reg(),
                    SCX_REG => self.ppu.read_scx_reg(),
                    LY_REG => self.ppu.read_ly_reg(),
                    LYC_REG => self.ppu.read_lyc_reg(),
                    BGP_REG => self.ppu.read_bgp_reg(),
                    OBP0_REG => self.ppu.read_obp0_reg(),
                    OBP1_REG => self.ppu.read_obp1_reg(),
                    WY_REG => self.ppu.read_wy_reg(),
                    WX_REG => self.ppu.read_wx_reg(),
                    INTERRUPT_FLAG_REG => self.interrupt_handler.read_if_reg(),
                    DMA => self.dma.read_source_address(),
                    _ => self.io[(address - IO_START) as usize],
                } 
            }
            HRAM_START ..= HRAM_END => self.hram[(address - HRAM_START) as usize],
            INTERRUPT_ENABLE_START => self.interrupt_handler.read_ie_reg(),
        }
    }

    pub fn write_byte(&mut self, address: u16, data_to_write: u8) {
        //Can't write anything below OAM while DMA is going
        if self.dma.currently_transferring && address < OAM_START {
            return;
        }

        match address {
            RAM_ENABLE_START ..= RAM_ENABLE_END => self.ram_enable_reg = (data_to_write & 0xF) == 0xA,
            ROM_BANK_NUM_START ..= ROM_BANK_NUM_END => {
                let bank_size = 
                self.rom_bank_num_reg = (data_to_write & 0x1F) | 0x1
                
            },
            VRAM_START ..= VRAM_END => {
                if self.ppu.current_mode() != PpuMode::DrawingPixels || !self.ppu.is_active() {
                    match address {
                        TILE_DATA_0_START ..= TILE_DATA_0_END => self.ppu.write_tile_data_0(address, data_to_write),
                        TILE_DATA_1_START ..= TILE_DATA_1_END => self.ppu.write_tile_data_1(address, data_to_write),
                        TILE_DATA_2_START ..= TILE_DATA_2_END => self.ppu.write_tile_data_2(address, data_to_write),
                        TILE_MAP_0_START ..= TILE_MAP_0_END => self.ppu.write_tile_map_0(address, data_to_write),
                        TILE_MAP_1_START ..= TILE_MAP_1_END => self.ppu.write_tile_map_1(address, data_to_write),
                        _ => panic!("MEMORY WRITE ERROR: Should have never gotten here since we took care of all the VRAM addresses"),
                    }
                }
            },
            SRAM_START ..= SRAM_END => self.sram[(address - SRAM_START) as usize] = data_to_write,
            WRAM_0_START ..= WRAM_0_END => self.wram_0[(address - WRAM_0_START) as usize] = data_to_write,
            WRAM_X_START ..= WRAM_X_END => self.wram_x[(address - WRAM_X_START) as usize] = data_to_write,
            ECHO_START ..= ECHO_END => {
                let wram_address = address - 0x2000;
                match wram_address { 
                    WRAM_0_START ..= WRAM_0_END => self.wram_0[(wram_address - WRAM_0_START) as usize] = data_to_write,
                    WRAM_X_START ..= WRAM_X_END => self.wram_x[(wram_address - WRAM_X_START) as usize] = data_to_write,
                    _ => panic!("Issues calculating echo ram")
                }
            }
            OAM_START ..= OAM_END => {
                if (self.ppu.current_mode() != PpuMode::OamScan && self.ppu.current_mode() != PpuMode::DrawingPixels) || !self.ppu.is_active() {
                    self.ppu.write_oam(address, data_to_write)
                }
            },
            UNUSED_START ..= UNUSED_END => {
                self.unused[(address - UNUSED_START) as usize] = data_to_write;
            }
            IO_START ..= IO_END => {
                match address {
                    JOYPAD_P1_REG => self.joypad.write_joypad_reg(data_to_write),
                    SERIAL_SB_REG => self.serial.write_sb_reg(data_to_write),
                    SERIAL_SC_REG => self.serial.write_sc_reg(data_to_write),
                    TIMER_DIV_REG => self.timer.write_2_div(),
                    TIMER_TIMA_REG => self.timer.write_2_tima(data_to_write),
                    TIMER_TMA_REG => self.timer.write_2_tma(data_to_write),
                    TIMER_TAC_REG => self.timer.write_2_tac(data_to_write),
                    LCDC_REG => self.ppu.write_lcdc_reg(data_to_write),
                    STAT_REG => self.ppu.write_stat_reg(data_to_write),
                    SCY_REG => self.ppu.write_scy_reg(data_to_write),
                    SCX_REG => self.ppu.write_scx_reg(data_to_write),
                    LY_REG => (),   //This is read only you can't touch it
                    LYC_REG => self.ppu.write_lyc_reg(data_to_write),
                    BGP_REG => self.ppu.write_bgp_reg(data_to_write),
                    OBP0_REG => self.ppu.write_obp0_reg(data_to_write),
                    OBP1_REG => self.ppu.write_obp1_reg(data_to_write),
                    WY_REG => self.ppu.write_wy_reg(data_to_write),
                    WX_REG => self.ppu.write_wx_reg(data_to_write),
                    INTERRUPT_FLAG_REG => self.interrupt_handler.write_if_reg(data_to_write),
                    DMA => self.dma.write_source_address(data_to_write),
                    _ => self.io[(address - IO_START) as usize] = data_to_write,
                } 
            }
            HRAM_START ..= HRAM_END => self.hram[(address - HRAM_START) as usize] = data_to_write,
            INTERRUPT_ENABLE_START => self.interrupt_handler.write_ie_reg(data_to_write),
        }
    }

    /**
     * Will carry out one cpu clk cycle for DMA. This is essentially
     * write to oam of the src address data
     */
    pub fn dma_cycle(&mut self) {
        match self.dma.cycle() {
            None => (),
            Some((src_address, oam_offset)) => {
                let oam_address = OAM_START + oam_offset as u16;
                let src_address_data = self.read_byte(src_address);
                self.write_byte(oam_address, src_address_data);
            },
        }
    }

    pub fn gpu_cycle(&mut self, buffer: &mut Vec<u32>, buffer_index: &mut usize) {
        if let Some(pixel_color) = self.ppu.cycle() {
            buffer[*buffer_index] = match pixel_color {
                super::ppu::enums::PaletteColors::White => 0xFFFFFF,
                super::ppu::enums::PaletteColors::LightGrey => 0xC0C0C0,
                super::ppu::enums::PaletteColors::DarkGrey => 0x606060,
                super::ppu::enums::PaletteColors::Black => 0x0,
            };
            *buffer_index += 1;
        }

        if self.ppu.vblank_interrupt_req {
            self.ppu.vblank_interrupt_req = false;
            self.interrupt_handler.if_reg |= 0x1;
        }

        if self.ppu.stat_interrupt_req {
            self.interrupt_handler.if_reg |= 0x2;
        }
    }

    pub fn timer_cycle(&mut self) {
        self.timer.cycle();
        if self.timer.interrupted_requested {
            self.interrupt_handler.if_reg |= 0x04;
            self.timer.interrupted_requested = false;
        }
    }

    /**
     * Again this is wildly ugly but I had to pull out the memory read and writes because they aren't avaliable
     * to the interrupt handler, but its also apart of the memory object so I can't pass it in. I'm going to have
     * to refactor this later
     */
    pub fn interrupt_cycle(&mut self, pc: &mut u16, sp: &mut u16) {
        match self.interrupt_handler.cycle(pc) {
            3 => {
                *sp = (*sp).wrapping_sub(1);
                self.write_byte(*sp, (*pc >> 8) as u8);
            },
            4 => {
                *sp = (*sp).wrapping_sub(1);
                self.write_byte(*sp, *pc as u8);
            },
            _ => (), 
        }
    }


    pub fn copy_game_data_to_rom(&mut self, bank_0: [u8; 0x4000], bank_1: [u8; 0x4000]) {
        self.rom_bank_0 = bank_0;
        self.rom_bank_x = bank_1;
    }
}