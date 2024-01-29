use crate::gameboy::{ppu::enums::*, binary_utils};

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
pub struct LcdcReg {
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
            win_tile_map_area: TileMapArea::_9800_9BFF,
            win_enable: State::Off,
            bg_win_tile_data_area: TileDataArea::_8000_8FFF,
            bg_tile_map_area: TileMapArea::_9800_9BFF,
            sprite_size: SpriteSize::_8x8,
            sprite_enable: State::Off,
            bg_win_priority: State::Off,
        }
    }

    pub fn read_reg_raw(&self) -> u8 {
        let mut value = 0;

        value |= match self.lcd_ppu_enable {
            State::Off => 0,
            State::On => 1 << 7,
        };

        value |= match self.win_tile_map_area {
            TileMapArea::_9800_9BFF => 0,
            TileMapArea::_9C00_9FFF => 1 << 6,
        };

        value |= match self.win_enable {
            State::Off => 0,
            State::On => 1 << 5,
        };

        value |= match self.bg_win_tile_data_area {
            TileDataArea::_8800_97FF => 0,
            TileDataArea::_8000_8FFF => 1 << 4,
        };

        value |= match self.bg_tile_map_area {
            TileMapArea::_9800_9BFF => 0,
            TileMapArea::_9C00_9FFF => 1 << 3,
        };

        value |= match self.sprite_size {
            SpriteSize::_8x8 => 0,
            SpriteSize::_8x16 => 1 << 2,
        };

        value |= match self.sprite_enable {
            State::Off => 0,
            State::On => 1 << 1
        };

        value |= match self.bg_win_priority {
            State::Off => 0,
            State::On => 1 << 0,
        };

        return value;
    }

    pub fn write_reg_raw(&mut self, value: u8) {
        self.lcd_ppu_enable = match binary_utils::get_bit(value, 7) {
            0 => State::Off,
            1 => State::On,
            _ => panic!("uh not 0 or 1"),
        };

        self.win_tile_map_area = match binary_utils::get_bit(value, 6) {
            0 => TileMapArea::_9800_9BFF,
            1 => TileMapArea::_9C00_9FFF,
            _ => panic!("uh not 0 or 1"),
        };

        self.win_enable = match binary_utils::get_bit(value, 5) {
            0 => State::Off,
            1 => State::On,
            _ => panic!("uh not 0 or 1"),
        };

        self.bg_win_tile_data_area = match binary_utils::get_bit(value, 4) {
            0 => TileDataArea::_8800_97FF,
            1 => TileDataArea::_8000_8FFF,
            _ => panic!("uh not 0 or 1"),
        };

        self.bg_tile_map_area = match binary_utils::get_bit(value, 3) {
            0 => TileMapArea::_9800_9BFF,
            1 => TileMapArea::_9C00_9FFF,
            _ => panic!("uh not 0 or 1"),
        };

        self.sprite_size = match binary_utils::get_bit(value, 2) {
            0 => SpriteSize::_8x8,
            1 => SpriteSize::_8x16,
            _ => panic!("uh not 0 or 1"),
        };

        self.sprite_enable = match binary_utils::get_bit(value, 1) {
            0 => State::Off,
            1 => State::On,
            _ => panic!("uh not 0 or 1"),
        };

        self.bg_win_priority = match binary_utils::get_bit(value, 0) {
            0 => State::Off,
            1 => State::On,
            _ => panic!("uh not 0 or 1"),
        };
    }
}

pub struct StatReg {
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

    pub fn read_reg_raw(&self) -> u8 {
        let mut value = 0;

        value |= match self.lyc_int_select {
            State::Off => 0,
            State::On => 1 << 6,
        };

        value |= match self.mode_2_int_select {
            State::Off => 0,
            State::On => 1 << 5,
        };

        value |= match self.mode_1_int_select {
            State::Off => 0,
            State::On => 1 << 4,
        };

        value |= match self.mode_0_int_select {
            State::Off => 0,
            State::On => 1 << 3,
        };

        value |= match self.lyc_ly_compare {
            State::Off => 0,
            State::On => 1 << 2,
        };

        value |= match self.ppu_mode {
            PpuMode::Hblank => 0b00,
            PpuMode::Vblank => 0b01,
            PpuMode::OamScan => 0b10,
            PpuMode::DrawingPixels => 0b11,
        };

        return value;
    }

    /**
     * NOTE we cannot write to the lyc==ly and ppumode bits since they are readonly
     */
    pub fn write_reg_from_u8(&mut self, value: u8) {
        self.lyc_int_select = match binary_utils::get_bit(value, 6) {
            0 => State::Off,
            1 => State::On,
            _ => panic!("suppose to be either 0 or 1"),
        };
        self.mode_2_int_select = match binary_utils::get_bit(value, 5) {
            0 => State::Off,
            1 => State::On,
            _ => panic!("suppose to be either 0 or 1"),
        };
        self.mode_1_int_select = match binary_utils::get_bit(value, 4) {
            0 => State::Off,
            1 => State::On,
            _ => panic!("suppose to be either 0 or 1"),
        };
        self.mode_0_int_select = match binary_utils::get_bit(value, 3) {
            0 => State::Off,
            1 => State::On,
            _ => panic!("suppose to be either 0 or 1"),
        };
    }
}

/**
 * Represents a register that contains color id for palettes. This can be used 
 * for object and background palette registers
 */
pub struct PaletteReg {
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

    pub fn write_reg_from_u8(&mut self, value: u8) {
        self.color_id_0 = match (binary_utils::get_bit(value, 1), binary_utils::get_bit(value, 0)) {
            (0,0) => PaletteColors::White,
            (0,1) => PaletteColors::LightGrey,
            (1,0) => PaletteColors::DarkGrey,
            (1,1) => PaletteColors::Black,
            _ => panic!("weird as bit combo"),
        };
        
        self.color_id_1 = match (binary_utils::get_bit(value, 3), binary_utils::get_bit(value, 2)) {
            (0,0) => PaletteColors::White,
            (0,1) => PaletteColors::LightGrey,
            (1,0) => PaletteColors::DarkGrey,
            (1,1) => PaletteColors::Black,
            _ => panic!("weird as bit combo"),
        };

        self.color_id_2 = match (binary_utils::get_bit(value, 5), binary_utils::get_bit(value, 4)) {
            (0,0) => PaletteColors::White,
            (0,1) => PaletteColors::LightGrey,
            (1,0) => PaletteColors::DarkGrey,
            (1,1) => PaletteColors::Black,
            _ => panic!("weird as bit combo"),
        };

        self.color_id_3 = match (binary_utils::get_bit(value, 7), binary_utils::get_bit(value, 6)) {
            (0,0) => PaletteColors::White,
            (0,1) => PaletteColors::LightGrey,
            (1,0) => PaletteColors::DarkGrey,
            (1,1) => PaletteColors::Black,
            _ => panic!("weird as bit combo"),
        };
    }

    /**
     * Returns the colors as a u8. Note for Sprites if the colorID 0 is always
     * going to mean  transparent 
     */
    pub fn read_reg_raw(&self) -> u8 {
        let mut value = 0;

        value |= match self.color_id_0 {
            PaletteColors::White => 0b00, //doesn't really matter here
            PaletteColors::LightGrey => 0b01,   
            PaletteColors::DarkGrey => 0b10,
            PaletteColors::Black => 0b11,
        };

        value |= match self.color_id_1 {
            PaletteColors::White => 0b00,   //doesn't really matter here
            PaletteColors::LightGrey => 0b01 << 2,   
            PaletteColors::DarkGrey => 0b10 << 2,
            PaletteColors::Black => 0b11 << 2,
        };

        value |= match self.color_id_2 {
            PaletteColors::White => 0b00,   //doesn't really matter here
            PaletteColors::LightGrey => 0b01 << 4,   
            PaletteColors::DarkGrey => 0b10 << 4,
            PaletteColors::Black => 0b11 << 4,
        };

        value |= match self.color_id_3 {
            PaletteColors::White => 0b00,   //doesn't really matter here
            PaletteColors::LightGrey => 0b01 << 6,   
            PaletteColors::DarkGrey => 0b10 << 6,
            PaletteColors::Black => 0b11 << 6,
        };

        return value;
    }
}