use crate::gameboy::binary_utils;

use super::{enums::{PaletteColors, SpritePalette, SpritePriority, TileMapArea}, registers::{LcdcReg, PpuRegisters}};

/**
 * Represents the pixel fetcher in the gameboy. It'll house all the things 
 * necessary to fetch sprite, bg, and window pixels
 */
pub struct PixelFetcher {
    x_coordinate: u8,   //Always between 0 and 31
    y_coordinate: u8,   //Always between 0 and 255
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
    
    /**
     * Does the entire process of fetching a tile
     */
    pub fn fetch_pixel_row(&mut self, ppu_registers: &PpuRegisters, tile_map_9800: &[u8; 0x400], tile_map_9c00: &[u8; 0x400]) {
        let tile_map = match self.determine_tile_map(ppu_registers) {
            TileMapArea::_9800_9BFF => tile_map_9800,
            TileMapArea::_9C00_9FFF => tile_map_9c00,
        };

        let tile_idx



        self.x_coordinate += 1;
    }

    /**
     * Returns the enum of the tile map that we are using
     */
    fn determine_tile_map(&self, ppu_registers: &PpuRegisters) -> TileMapArea {
        let lcdc_reg = &ppu_registers.lcdc;

        if (lcdc_reg.bg_tile_map_area == TileMapArea::_9C00_9FFF && 
            (ppu_registers.x_scanline_coord + 7 < ppu_registers.wx || ppu_registers.ly < ppu_registers.wy)) || 
                (lcdc_reg.win_tile_map_area == TileMapArea::_9C00_9FFF && 
                    ppu_registers.x_scanline_coord + 7 >= ppu_registers.wx && ppu_registers.ly >= ppu_registers.wy) {
            return TileMapArea::_9C00_9FFF;            
        }
        return TileMapArea::_9800_9BFF;
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