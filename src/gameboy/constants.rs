pub const ROM_BANK_0_START: u16 = 0x0000;
pub const ROM_BANK_0_END: u16 = 0x3FFF;
pub const ROM_BANK_X_START: u16 = 0x4000;
pub const ROM_BANK_X_END: u16 = 0x7FFF;
pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
pub const SRAM_START: u16 = 0xA000;
pub const SRAM_END: u16 = 0xBFFF;
pub const WRAM_0_START: u16 = 0xC000;
pub const WRAM_0_END: u16 = 0xCFFF;
pub const WRAM_X_START: u16 = 0xD000;
pub const WRAM_X_END: u16 = 0xDFFF;
pub const ECHO_START: u16 = 0xE000;
pub const ECHO_END: u16 = 0xFDFF;
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;
pub const UNUSED_START: u16 = 0xFEA0;
pub const UNUSED_END: u16 = 0xFEFF;
pub const IO_START: u16 = 0xFF00;
pub const IO_END: u16 = 0xFF7F;
pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;
pub const INTERRUPT_ENABLE_START: u16 = 0xFFFF;

pub const TIMER_START: u16 = 0xFF04;
pub const TIMER_END: u16 = 0xFF07;
pub const TIMER_DIV_REG: u16 = 0xFF04;
pub const TIMER_TIMA_REG: u16 = 0xFF05;
pub const TIMER_TMA_REG: u16 = 0xFF06;
pub const TIMER_TAC_REG: u16 = 0xFF07;
pub const INTERRUPT_FLAG_REG: u16 = 0xFF0F;

pub const TILE_DATA_0_START: u16 = 0x8000;
pub const TILE_DATA_0_END: u16 = 0x87FF;
pub const TILE_DATA_1_START: u16 = 0x8800;
pub const TILE_DATA_1_END: u16 = 0x8FFF;
pub const TILE_DATA_2_START: u16 = 0x9000;
pub const TILE_DATA_2_END: u16 = 0x97FF;
pub const TILE_MAP_0_START: u16 = 0x9800;
pub const TILE_MAP_0_END: u16 = 0x9BFF;
pub const TILE_MAP_1_START: u16 = 0x9C00;
pub const TILE_MAP_1_END: u16 = 0x9FFF;

pub const LCDC_REG: u16 = 0xFF40;
pub const STAT_REG: u16 = 0xFF41;
pub const SCY_REG: u16 = 0xFF42;
pub const SCX_REG: u16 = 0xFF43;
pub const LY_REG: u16 = 0xFF44;
pub const LYC_REG: u16 = 0xFF45;
pub const BGP_REG: u16 = 0xFF47;
pub const OBP0_REG: u16 = 0xFF48;
pub const OBP1_REG: u16 = 0xFF49;
pub const WY_REG: u16 = 0xFF4A;
pub const WX_REG: u16 = 0xFF4B;

pub const DMA: u16 = 0xFF46;

pub const JOYPAD_P1_REG: u16 = 0xFF00;

pub const SERIAL_SB_REG: u16 = 0xFF01;
pub const SERIAL_SC_REG: u16 = 0xFF02;

pub const MACHINE_CYCLE: u8 = 4;
pub const PREFIX_OPCODE: u8 = 0xCB;
pub const MAX_SCANLINE_CLK_TICKS: u16 = 456;
pub const MAX_LY_VALUE: u8 = 153;

//Constants that are just zero
pub const LOWEST_PRIORITY_BG_COLOR: u8 = 0;
pub const TRANSPARENT: u8 = 0;
