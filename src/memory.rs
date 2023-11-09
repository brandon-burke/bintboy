use crate::timer::Timer;
use crate::ppu::Ppu;
use crate::interrupt_handler::InterruptHandler;
use crate::constants::*;

pub struct Memory {
    rom_bank_0: [u8; 0x4000],   //16KB -> 0000h – 3FFFh (Non-switchable ROM bank)
    rom_bank_x: [u8; 0x4000],   //16KB -> 4000h – 7FFFh (Switchable ROM bank)
    sram: [u8; 0x2000],         //8KB  -> A000h – BFFFh (External RAM in cartridge)
    wram_0: [u8; 0x1000],       //1KB  -> C000h – CFFFh (Work RAM)
    wram_x: [u8; 0x1000],       //1KB  -> D000h – DFFFh (Work RAM)
    echo: [u8; 0x1E00],         //     -> E000h – FDFFh (ECHO RAM) Mirror of C000h-DDFFh
    unused: [u8; 0x60],         //     -> FEA0h – FEFFh (Unused)
    io: [u8; 0x80],             //     -> FF00h – FF7Fh (I/O ports)
    pub interrupt_handler: InterruptHandler, //Will contain IE, IF, and IME registers (0xFFFF, 0xFF0F)
    ppu: Ppu,                   //Pixel Processing Unit. Houses most of the graphics related memory 
    timer: Timer,               //     -> FF04 - FF07
    hram: [u8; 0x7F],           //     -> FF80h – FFFEh (HRAM)
}

impl Memory {
    pub fn new() -> Self {
        Self {
            rom_bank_0: [0; 0x4000],   
            rom_bank_x: [0; 0x4000],         
            sram: [0; 0x2000],        
            wram_0: [0; 0x1000],       
            wram_x: [0; 0x1000],   
            echo: [0; 0x1E00],                 
            unused: [0; 0x60],
            io: [0; 0x80],
            interrupt_handler: InterruptHandler::new(), 
            timer: Timer::new(),              
            ppu: Ppu::new(),
            hram: [0; 0x7F],            
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_START ..= ROM_BANK_0_END => self.rom_bank_0[address as usize],
            ROM_BANK_X_START ..= ROM_BANK_X_END => self.rom_bank_x[(address - ROM_BANK_X_START) as usize],
            VRAM_START ..= VRAM_END => {
                match address {
                    TILE_DATA_0_START ..= TILE_DATA_0_END => self.ppu.read_tile_data_0(address),
                    TILE_DATA_1_START ..= TILE_DATA_1_END => self.ppu.read_tile_data_1(address),
                    TILE_DATA_2_START ..= TILE_DATA_2_END => self.ppu.read_tile_data_2(address),
                    TILE_MAP_0_START ..= TILE_MAP_0_END => self.ppu.read_tile_map_0(address),
                    TILE_MAP_1_START ..= TILE_MAP_1_END => self.ppu.read_tile_map_1(address),
                    _ => panic!("MEMORY READ ERROR: Should have never gotten here since we took care of all the VRAM addresses"),
                }
            },
            SRAM_START ..= SRAM_END => self.sram[(address - SRAM_START) as usize],
            WRAM_0_START ..= WRAM_0_END => self.wram_0[(address - WRAM_0_START) as usize],
            WRAM_X_START ..= WRAM_X_END => self.wram_x[(address - WRAM_X_START) as usize],
            ECHO_START ..= ECHO_END => {
                panic!("I don't think we should be accessing echo memory");
            }
            OAM_START ..= OAM_END => self.ppu.read_oam(address),
            UNUSED_START ..= UNUSED_END => {
                panic!("I don't think we should be accessing unused memory");
            }
            IO_START ..= IO_END => {
                match address {
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
                    _ => self.io[(address - IO_START) as usize],
                } 
            }
            HRAM_START ..= HRAM_END => self.hram[(address - HRAM_START) as usize],
            INTERRUPT_ENABLE_START => self.interrupt_handler.read_ie_reg(),
        }
    }

    pub fn write_byte(&mut self, address: u16, data_to_write: u8) {
        match address {
            ROM_BANK_0_START ..= ROM_BANK_0_END => self.rom_bank_0[address as usize] = data_to_write,
            ROM_BANK_X_START ..= ROM_BANK_X_END => self.rom_bank_x[(address - ROM_BANK_X_START) as usize] = data_to_write,
            VRAM_START ..= VRAM_END => {
                match address {
                    TILE_DATA_0_START ..= TILE_DATA_0_END => self.ppu.write_tile_data_0(address, data_to_write),
                    TILE_DATA_1_START ..= TILE_DATA_1_END => self.ppu.write_tile_data_1(address, data_to_write),
                    TILE_DATA_2_START ..= TILE_DATA_2_END => self.ppu.write_tile_data_2(address, data_to_write),
                    TILE_MAP_0_START ..= TILE_MAP_0_END => self.ppu.write_tile_map_0(address, data_to_write),
                    TILE_MAP_1_START ..= TILE_MAP_1_END => self.ppu.write_tile_map_1(address, data_to_write),
                    _ => panic!("MEMORY WRITE ERROR: Should have never gotten here since we took care of all the VRAM addresses"),
                }
            },
            SRAM_START ..= SRAM_END => self.sram[(address - SRAM_START) as usize] = data_to_write,
            WRAM_0_START ..= WRAM_0_END => self.wram_0[(address - WRAM_0_START) as usize] = data_to_write,
            WRAM_X_START ..= WRAM_X_END => self.wram_x[(address - WRAM_X_START) as usize] = data_to_write,
            ECHO_START ..= ECHO_END => {
                panic!("I don't think we should be writing echo memory");
            }
            OAM_START ..= OAM_END => self.ppu.write_oam(address, data_to_write),
            UNUSED_START ..= UNUSED_END => {
                panic!("I don't think we should be writing unused memory");
            }
            IO_START ..= IO_END => {
                match address {
                    TIMER_DIV_REG => self.timer.write_2_div(),
                    TIMER_TIMA_REG => self.timer.write_2_tima(data_to_write),
                    TIMER_TMA_REG => self.timer.write_2_tma(data_to_write),
                    TIMER_TAC_REG => self.timer.write_2_tac(data_to_write),
                    LCDC_REG => self.ppu.write_lcdc_reg(data_to_write),
                    STAT_REG => self.ppu.write_stat_reg(data_to_write),
                    SCY_REG => self.ppu.write_scy_reg(data_to_write),
                    SCX_REG => self.ppu.write_scx_reg(data_to_write),
                    LY_REG => self.ppu.write_ly_reg(data_to_write),
                    LYC_REG => self.ppu.write_lyc_reg(data_to_write),
                    BGP_REG => self.ppu.write_bgp_reg(data_to_write),
                    OBP0_REG => self.ppu.write_obp0_reg(data_to_write),
                    OBP1_REG => self.ppu.write_obp1_reg(data_to_write),
                    WY_REG => self.ppu.write_wy_reg(data_to_write),
                    WX_REG => self.ppu.write_wx_reg(data_to_write),
                    INTERRUPT_FLAG_REG => self.interrupt_handler.write_if_reg(data_to_write),
                    _ => self.io[(address - IO_START) as usize] = data_to_write,
                } 
            }
            HRAM_START ..= HRAM_END => self.hram[(address - HRAM_START) as usize] = data_to_write,
            INTERRUPT_ENABLE_START => self.interrupt_handler.write_ie_reg(data_to_write),
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

    pub fn load_rom(&mut self, rom_0: [u8; 0x4000], rom_1: [u8; 0x4000]) {
        self.rom_bank_0 = rom_0;
        self.rom_bank_x = rom_1;
    }
}