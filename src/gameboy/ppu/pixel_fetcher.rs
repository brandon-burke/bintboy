

use std::collections::VecDeque;

use crate::gameboy::binary_utils;

use super::{enums::{PaletteColors, SpritePalette, SpritePriority, TileDataArea, TileMapArea}, registers::PpuRegisters, Tile};

/**
 * Represents the pixel fetcher in the gameboy. It'll house all the things 
 * necessary to fetch sprite, bg, and window pixels
 */
pub struct PixelFetcher {
    x_coordinate: u8,           //Gives the x TILE coordinate on the 32x32 tile map. Value between 0-31
    drawing_window: bool,       //Lets us know if we are rendering the window
    transitioned: bool,         //Tells us if we switch from bg tiles and windows tiles
}

impl PixelFetcher {
    pub fn new() -> Self {
        Self {
            x_coordinate: 0,
            drawing_window: false,
            transitioned: false,
        }
    }
    
    /**
     * Does the entire process of fetching a tile
     */
    pub fn fetch_pixel_row(&mut self, ppu_registers: &PpuRegisters, tile_map: &[u8], 
                                    tile_data_map_0: &[Tile], tile_data_map_1: &[Tile]) -> Vec<Pixel> {           
        let mut number_of_pixels_to_discard = 0;

        //Toggling flag if we are transitioning from bg to window or vice versa
        if self.bg_or_win_transition(ppu_registers) {
            self.drawing_window = !self.drawing_window;
            self.x_coordinate = ((ppu_registers.scx / 8) + (ppu_registers.x_scanline_coord / 8)) & 0x1F;
            
            unimplemented!("May need to implement discarding some pixels since we 
                            may be pausing rendering in the middle of a tile. So that means we really
                            don't need the entire 8 pixel row");
        }

        //Getting all info to index into the tile map and tile data map
        let tile_map_x_coord = ((ppu_registers.scx / 8) + self.x_coordinate) & 0x1F;
        let tile_map_y_coord = (ppu_registers.ly + ppu_registers.scy) & 255;
        let tile_map_idx = tile_map_x_coord + ((tile_map_y_coord / 8) * 32);
        let tile_data_idx = tile_map[tile_map_idx as usize];

        //Just getting the actual tile now
        let tile = match tile_data_idx {
            0..=127 => tile_data_map_0[tile_data_idx as usize],
            128..=255 => tile_data_map_1[(tile_data_idx - 128) as usize],
        };

        //Figuring out what row of pixels we need to get
        let row_idx = tile_map_y_coord - ((tile_map_y_coord / 8) * 8);
        let tile_row = tile.pixel_rows[row_idx as usize];

        //Now constructing the row of pixels to be sent to the bg/window fifo
        let mut constructed_pixels: Vec<Pixel> = vec![];
        for bit_pos in (0..8).rev() {
            //Grouping the lsb and msb
            let color_id = binary_utils::get_bit(tile_row.upper_bits, bit_pos) << 1 |
                            binary_utils::get_bit(tile_row.lower_bits, bit_pos);
            
            //Constructing the pixel
            let new_pixel = Pixel {
                color_id,
                palette: None,
                bg_priority: None,
            };

            constructed_pixels.push(new_pixel);
        }
        self.x_coordinate += 1;

        //Need to pop off some pixels since we could be in the middle of the tile drawing from last time
        if self.transitioned {

            constructed_pixels.remove(index);
        }

        return constructed_pixels;
    }

    /**
     * Returns the enum of the tile map that we are using
     */
    pub fn determine_tile_map(&self, ppu_registers: &PpuRegisters) -> TileMapArea {
        let lcdc_reg = &ppu_registers.lcdc;

        if (lcdc_reg.bg_tile_map_area == TileMapArea::_9C00_9FFF && !self.is_inside_window(ppu_registers)) 
            || (lcdc_reg.win_tile_map_area == TileMapArea::_9C00_9FFF && self.is_inside_window(ppu_registers)) {
            return TileMapArea::_9C00_9FFF;            
        }
        return TileMapArea::_9800_9BFF;
    }

    /**
     * Returns true if we are transitioning from drawing the bg to window or 
     * vice versa. This will also affect 
     */
    pub fn bg_or_win_transition(&self, ppu_registers: &PpuRegisters) -> bool {
        //Checking if we are rendering the window
        if self.is_inside_window(ppu_registers) && !self.drawing_window {
            return true;
        }

        if !self.is_inside_window(ppu_registers) && self.drawing_window {
            return true;
        }
        return false;
    }

    /**
     * Will return true if the current pixel that you are drawing is inside the 
     * window
     */
    fn is_inside_window(&self, ppu_registers: &PpuRegisters) {
       if ppu_registers.x_scanline_coord + 7 >= ppu_registers.wx && 
                ppu_registers.ly >= ppu_registers.wy {
            return true;
        }
       return false;
    }
}

#[derive(Clone, Copy)]
pub struct Pixel {
    color_id: u8,
    palette: Option<SpritePalette>,
    bg_priority: Option<SpritePriority>,
}

impl Pixel {
    pub fn new() -> Self {
        Self {
            color_id: 0,
            palette: None,
            bg_priority: None,
        }
    }
}