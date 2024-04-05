use crate::gameboy::ppu::enums::{SpritePriority, Orientation, SpritePalette, VramBank};
use crate::gameboy::binary_utils;

/*
    Represents a 8x8 square of pixels. Here we have an array of PixelRows
    where each PixelRow in the array represents the data of a row of pixels
    in the Tile. arr[0] being the first row and so on
 */
#[derive(Clone, Copy)]
pub struct Tile {
    pub pixel_rows: [TileRow; 8]
}

impl Tile {
    pub fn new() -> Self {
        Self {
            pixel_rows: [TileRow::new(); 8]
        }
    }
}

/**
 * Represents the data to create a row of pixels. In memory the lower bits
 * come first.
 */
#[derive(Clone, Copy)]
pub struct TileRow {
    pub lower_bits: u8, //lsb
    pub upper_bits: u8, //msb
}

impl TileRow {
    fn new() -> Self {
        Self {
            lower_bits: 0,
            upper_bits: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Sprite {
    pub y_pos: u8,
    pub x_pos: u8,
    pub tile_index: u8,
    pub priority: SpritePriority,
    pub y_flip: Orientation,
    pub x_flip: Orientation,
    pub dmg_palette: SpritePalette,     //Non CGB Mode only
    pub bank: VramBank,                 //CGB Mode only
    pub cgb_palette: SpritePalette      //CGB Mode only
}

impl Sprite {
    pub fn new() -> Self {
        Self {
            y_pos: 0,
            x_pos: 0,
            tile_index: 0,
            priority: SpritePriority::OverBg,
            y_flip: Orientation::Normal,
            x_flip: Orientation::Normal,
            dmg_palette: SpritePalette::Obp0,
            bank: VramBank::Bank0, 
            cgb_palette: SpritePalette::Obp0,
        }
    }

    /**
     * Just returns the attributes flag but in a u8 format where
     * each bit represents one of the options. This will always make the 
     * first 3 bits 0
     */
    pub fn attribute_flags_raw(&self) -> u8 {
        let mut value = 0;

        //You can use match statments here buddy

        if self.priority ==  SpritePriority::UnderBg {
            value = value | (0x1 << 7);
        }

        if self.y_flip == Orientation::Mirrored {
            value = value | (0x1 << 6);
        }
        
        if self.x_flip == Orientation::Mirrored {
            value = value | (0x1 << 5);
        }

        if self.dmg_palette == SpritePalette::Obp1 {
            value = value | (0x1 << 4);
        }

        return value;
    }

    pub fn write_attribute_flags(&mut self, value: u8) {
        self.priority = match binary_utils::get_bit(value, 7) {
            0 => SpritePriority::OverBg,
            _ => SpritePriority::UnderBg,
        };

        self.y_flip = match binary_utils::get_bit(value, 6) {
            0 => Orientation::Normal,
            _ => Orientation::Mirrored,
        };

        self.x_flip = match binary_utils::get_bit(value, 6) {
            0 => Orientation::Normal,
            _ => Orientation::Mirrored,
        };

        self.dmg_palette = match binary_utils::get_bit(value, 5) {
            0 => SpritePalette::Obp0,
            _ => SpritePalette::Obp1,
        }
    }
}