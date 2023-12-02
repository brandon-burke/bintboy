use crate::gameboy::ppu::enums::*;

pub struct PpuRegisters {
    pub lcdc: LcdcReg,      //$FF40 - LCD Control register
    pub ly: u8,             //READ-ONLY -> $FF44 - LCD y coordinate register (current horizontal line which might be able to be drawn, being drawn, or just been drawn)
    pub lyc: u8,            //$FF45 - LY compare register. Can use this register to trigger an interrupt when LY reg and this reg are the same value
    pub stat: StatReg,      //$FF41 - LCD status register
    pub scx: u8,            //$FF43 - Scrolling x register
    pub scy: u8,            //$FF42 - Scrolling y register
    pub wx: u8,             //$FF4B - Window x position
    pub wy: u8,             //$FF4A - Window y position
    pub bgp: PaletteReg,    //$FF47 - Background palette data - Non-CGB Mode only
    pub obp0: PaletteReg,   //$FF48 - Object palette 0 data - Non-CGB Mode only
    pub obp1: PaletteReg,   //$FF49 - Object palette 1 data - Non-CGB Mode only
}

impl PpuRegisters {
    pub fn new() -> Self {
        Self {
            lcdc: LcdcReg::new(),
            ly: 0,
            lyc: 0,
            stat: StatReg::new(),
            scx: 0,
            scy: 0,
            wx: 0,
            wy: 0,
            bgp: PaletteReg::new(),
            obp0: PaletteReg::new(),
            obp1: PaletteReg::new(),
        }
    }

    pub fn sprite_size(&self) -> SpriteSize {
        return self.lcdc.sprite_size;
    }

    pub fn set_mode(&mut self, new_mode: PpuMode) {
        self.stat.ppu_mode = new_mode;
    }
}

/* Represents the LCD Control register (LCDC) */
struct LcdcReg {
    lcd_ppu_enable: State,
    win_tile_map_area: TileMapArea,
    win_enable: State,
    bg_win_tile_data_area: TileDataArea,
    bg_tile_map_area: TileMapArea,
    sprite_size: SpriteSize,
    sprite_enable: State,
    bg_win_priority: State,
}

impl LcdcReg {
    fn new() -> Self {
        Self {
            lcd_ppu_enable: State::Off,
            win_tile_map_area: TileMapArea::_9800,
            win_enable: State::Off,
            bg_win_tile_data_area: TileDataArea::_8000,
            bg_tile_map_area: TileMapArea::_9800,
            sprite_size: SpriteSize::_8x8,
            sprite_enable: State::Off,
            bg_win_priority: State::Off,
        }
    }
}

struct StatReg {
    pub unused_bit_7: u8,
    pub lyc_int_select: State,
    pub mode_2_int_select: State,
    pub mode_1_int_select: State,
    pub mode_0_int_select: State,
    pub lyc_ly_compare: State,      //Read-Only
    pub ppu_mode: PpuMode,          //Read-Only
}

impl StatReg {
    fn new() -> Self {
        Self {
            unused_bit_7: 0,
            lyc_int_select: State::Off,
            mode_2_int_select: State::Off,
            mode_1_int_select: State::Off,
            mode_0_int_select: State::Off,
            lyc_ly_compare: State::Off,
            ppu_mode: PpuMode::OamScan,
        }
    }
}

/**
 * Represents a register that contains color id for palettes. This can be used 
 * for object and background palette registers
 */
struct PaletteReg {
    color_id_0: PaletteColors,
    color_id_1: PaletteColors,
    color_id_2: PaletteColors,
    color_id_3: PaletteColors,
}

impl PaletteReg {
    fn new() -> Self {
        Self {
            color_id_0: PaletteColors::White,
            color_id_1: PaletteColors::LightGrey,
            color_id_2: PaletteColors::DarkGrey,
            color_id_3: PaletteColors::Black,
        }
    }
}