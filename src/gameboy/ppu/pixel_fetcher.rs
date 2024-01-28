use super::enums::{PaletteColors, SpritePalette, SpritePriority};

/**
 * Represents the pixel fetcher in the gameboy. It'll house all the things 
 * necessary to fetch sprite, bg, and window pixels
 */
pub struct PixelFetcher {
    x_coordinate: u8,
    y_coordinate: u8,
    fetched_pixel_row: [Pixel; 8],
}

impl PixelFetcher {
    pub fn new() -> Self {
        Self {
            x_coordinate: 0,
            y_coordinate: 0,
            fetched_pixel_row: [Pixel::new(); 8],
        }
    }
}

#[derive(Clone, Copy)]
pub struct Pixel {
    color: PaletteColors,
    palette: Option<SpritePalette>,
    bg_priority: Option<SpritePriority>,
}

impl Pixel {
    pub fn new() -> Self {
        Self {
            color: PaletteColors::White,
            palette: None,
            bg_priority: None,
        }
    }
}