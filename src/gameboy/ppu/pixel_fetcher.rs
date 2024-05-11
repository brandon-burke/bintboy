use crate::gameboy::binary_utils;

use super::{enums::{Orientation, SpritePalette, SpritePriority, SpriteSize, State, TileMapArea}, registers::PpuRegisters, Sprite, Tile};

/**
 * Represents the pixel fetcher in the gameboy. It'll house all the things 
 * necessary to fetch sprite, bg, and window pixels
 */
pub struct PixelFetcher {
    pub x_coordinate: u8,           //Gives the x TILE coordinate on the 32x32 tile map. Value between 0-31
    pub win_x_coordinate: u8,
    pub win_y_coordinate: u8,
    pub drawing_window: bool,       //Lets us know if we are rendering the window
}

impl PixelFetcher {
    pub fn new() -> Self {
        Self {
            x_coordinate: 0,
            win_x_coordinate: 0,
            win_y_coordinate: 0,
            drawing_window: false,
        }
    }

    pub fn early_transition(&mut self, ppu_registers: &PpuRegisters) {
        self.drawing_window = !self.drawing_window;
        self.x_coordinate = ((ppu_registers.scx / 8) + (ppu_registers.x_scanline_coord / 8)) & 0x1F;
    }
    
    /**
     * Does the entire process of fetching a pixel row
     */
    pub fn fetch_pixel_row(&mut self, ppu_registers: &PpuRegisters, tile_map: &[u8], 
                                    tile_data_map_0: &[Tile], tile_data_map_1: &[Tile]) -> Vec<Pixel> {           
        //Toggling flag if we are transitioning from bg to window or vice versa
        if self.bg_or_win_transition(ppu_registers) {
            self.drawing_window = !self.drawing_window;
            self.x_coordinate = ((ppu_registers.scx / 8) + (ppu_registers.x_scanline_coord / 8)) & 0x1F;
        }

        //Getting all info to index into the tile map and tile data map
        let (tile_map_x_coord, tile_map_y_coord) = if self.drawing_window {
            (self.win_x_coordinate, ppu_registers.ly - ppu_registers.wy)
        } else {
            (((ppu_registers.scx / 8) + self.x_coordinate) & 0x1F, ppu_registers.ly.wrapping_add(ppu_registers.scy))
        };

        let tile_map_idx = tile_map_x_coord as u16 + ((tile_map_y_coord as u16 / 8) * 32);
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
                is_sprite: false,
            };

            constructed_pixels.push(new_pixel);
        }
        
        //Making sure this value doesn't go above 31
        self.x_coordinate += 1;
        if self.x_coordinate > 31 { 
            self.x_coordinate = 0;
        }

        if self.drawing_window {
            self.win_x_coordinate += 1;
            if self.win_x_coordinate > 31 {
                self.win_x_coordinate = 0;
            }
        }

        return constructed_pixels;
    }

    /**
     * All this does is create a row of pixels of the sprite that we pass here.
     */
    pub fn fetch_sprite_pixel_row(&mut self, ppu_registers: &PpuRegisters, 
        tile_data_map_0: &[Tile], tile_data_map_1: &[Tile], sprite: &Sprite) -> Vec<Pixel> {

        //Checking which tile we should pick. Really only matters for 8x16 sprite mode
        let sprite_tile_index = match ppu_registers.sprite_size() {
            SpriteSize::_8x8 => sprite.tile_index,
            SpriteSize::_8x16 => {
                if ((ppu_registers.ly + 16) - sprite.y_pos) > 7 {   //The 7 is b/c the tile rows go from 0-7 not 1-8
                    if sprite.y_flip == Orientation::Normal {
                        sprite.tile_index | 0x01        //Enforcing to have a lsb
                    } else {
                        sprite.tile_index & 0xFE        //Enforcing to ignore the lsb
                    }
                } else {
                    if sprite.y_flip == Orientation::Normal {
                        sprite.tile_index & 0xFE        //Enforcing to ignore the lsb
                    } else {
                        sprite.tile_index | 0x01        //Enforcing to have a lsb
                    }
                }
            },
        };
    
        //Just getting the actual tile now
        let tile = match sprite_tile_index {
            0..=127 => tile_data_map_0[sprite_tile_index as usize],
            128..=255 => tile_data_map_1[(sprite_tile_index - 128) as usize],
        };

        //Figuring out what row of pixels we need to get. Accounting for flipping vertically
        let row_idx = match sprite.y_flip {
            Orientation::Normal => (ppu_registers.ly + 16) - sprite.y_pos,
            Orientation::Mirrored => {
                7 - ((ppu_registers.ly + 16) - sprite.y_pos)
            },
        };
        let tile_row = tile.pixel_rows[row_idx as usize];

        //Now constructing the row of pixels
        let mut constructed_pixels: Vec<Pixel> = vec![];
        for bit_pos in (0..8).rev() {
            //Grouping the lsb and msb
            let color_id = binary_utils::get_bit(tile_row.upper_bits, bit_pos) << 1 |
                            binary_utils::get_bit(tile_row.lower_bits, bit_pos);
            
            //Constructing the pixel
            let new_pixel = Pixel {
                color_id,
                palette: Some(sprite.dmg_palette),
                bg_priority: Some(sprite.priority),
                is_sprite: true,
            };

            constructed_pixels.push(new_pixel);
        }

        //Finally accounting for x flipping
        match sprite.x_flip {
            Orientation::Normal => (),
            Orientation::Mirrored => constructed_pixels.reverse(),
        }

        return constructed_pixels;
    }

    /**
     * Returns the enum of the tile map that we are using. This is also influenced
     * by the win enable bit. If the window is turned off we just default to the 
     * bg tile map.
     */
    pub fn determine_tile_map(&self, ppu_registers: &PpuRegisters) -> TileMapArea {
        let lcdc_reg = &ppu_registers.lcdc;

        //If the window is disabled just use the bg's tile map
        if ppu_registers.lcdc.win_enable == State::Off {
            return lcdc_reg.bg_tile_map_area;
        }

        if (lcdc_reg.bg_tile_map_area == TileMapArea::_9C00_9FFF && !self.is_inside_window(ppu_registers)) 
            || (lcdc_reg.win_tile_map_area == TileMapArea::_9C00_9FFF && self.is_inside_window(ppu_registers)) {
            return TileMapArea::_9C00_9FFF;            
        }
        return TileMapArea::_9800_9BFF;
    }

    /**
     * Returns true if we are transitioning from drawing the bg to window or 
     * vice versa. This also is influenced by the window enable bit in the lcdc
     * register.
     */
    pub fn bg_or_win_transition(&self, ppu_registers: &PpuRegisters) -> bool {
        //Checking bg to win 
        if ppu_registers.lcdc.win_enable == State::On && self.is_inside_window(ppu_registers) && !self.drawing_window {
            return true;
        }
        //Checking win to bg
        if !self.is_inside_window(ppu_registers) && self.drawing_window {
            return true;
        }
        //Checking win to win. We want to influence a change to bg b/c the window is not enabled
        if ppu_registers.lcdc.win_enable == State::Off && self.is_inside_window(ppu_registers) && self.drawing_window {
            return true;
        }

        return false;
    }

    /**
     * Since I'm too lazy to fix the original transition function. Here's one
     * that's specifically for when we go from bg to win
     */
    pub fn is_bg_to_win(&self, ppu_registers: &PpuRegisters) -> bool {
        if ppu_registers.lcdc.win_enable == State::On && self.is_inside_window(ppu_registers) && !self.drawing_window {
            return true;
        }
        return false;
    }

    /**
     * Will return true if the current pixel that you are drawing is inside the 
     * window
     */
    pub fn is_inside_window(&self, ppu_registers: &PpuRegisters) -> bool {
       if ppu_registers.x_scanline_coord + 7 >= ppu_registers.wx && 
                ppu_registers.ly >= ppu_registers.wy {
            return true;
        }
       return false;
    }
}

#[derive(Clone, Copy)]
pub struct Pixel {
    pub color_id: u8,
    pub palette: Option<SpritePalette>,
    pub bg_priority: Option<SpritePriority>,
    pub is_sprite: bool,
}

impl Pixel {
    /**
     * Creating a new sprite pixel with the lowest priority
     */
    pub fn new_translucent_sprite_pixel() -> Self {
        Self {
            color_id: 0,
            palette: Some(SpritePalette::Obp0),
            bg_priority: Some(SpritePriority::UnderBg),
            is_sprite: true,
        }
    }
}