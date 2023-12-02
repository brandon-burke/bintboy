use crate::gameboy::ppu::enums::{SpritePriority, Orientation, SpritePalette, VramBank};

/*
    Represents a 8x8 square of pixels. Here we have an array of PixelRows
    where each PixelRow in the array represents the data of a row of pixels
    in the Tile
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

/* Represents the data to create a row of pixels. */
#[derive(Clone, Copy)]
struct TileRow {
    lower_bits: u8,
    upper_bits: u8,
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
}