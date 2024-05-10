pub mod enums;
mod registers;
mod tile_and_sprite;
mod pixel_fetcher;

use self::pixel_fetcher::{Pixel, PixelFetcher};
use self::registers::PpuRegisters;
use self::enums::{PaletteColors, PpuMode, SpritePriority, SpriteScanlineVisibility, SpriteSize, State, TileDataArea};
use self::tile_and_sprite::*;
use crate::gameboy::constants::*;
pub struct Ppu {
    pub tile_data_0: [Tile; 128],       //$8000–$87FF
    pub tile_data_1: [Tile; 128],       //$8800–$8FFF
    pub tile_data_2: [Tile; 128],       //$9000–$97FF
    pub tile_map_0: [u8; 0x400],        //$9800-$9BFF
    pub tile_map_1: [u8; 0x400],        //$9C00-$9FFF
    pub oam: [Sprite; 40],              //$FE00–$FE9F (Object Attribute Table) Sprite information table
    ppu_registers: PpuRegisters,    //Houses all ppu registers
    clk_ticks: u16,                 //How many cpu ticks have gone by
    visible_sprites: Vec<Sprite>,   //Visible Sprites on current scanline
    pixel_fetcher: PixelFetcher,
    sprite_fifo: Vec<Pixel>,        
    bg_window_fifo: Vec<Pixel>,
    initial_pixel_shift: u8,
    penalty: u8,
    pub vblank_interrupt_req: bool,
    pub stat_interrupt_req: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            tile_data_0: [Tile::new(); 128],
            tile_data_1: [Tile::new(); 128],
            tile_data_2: [Tile::new(); 128],
            tile_map_0: [0; 0x400],
            tile_map_1: [0; 0x400],
            oam: [Sprite::new(); 40],
            ppu_registers: PpuRegisters::new(),
            clk_ticks: 0,
            visible_sprites: Vec::with_capacity(10),
            pixel_fetcher: PixelFetcher::new(),
            sprite_fifo: Vec::with_capacity(8),
            bg_window_fifo: Vec::with_capacity(16),
            initial_pixel_shift: 0,
            penalty: 0,
            vblank_interrupt_req: false,
            stat_interrupt_req: false,
        }
    }

    pub fn cycle(&mut self) -> Option<PaletteColors> {
        self.clk_ticks += 1;    //Keeps track of how many ticks during a mode

        match self.current_mode() {
            PpuMode::OamScan => {   //Mode 2
                
                //Finding up to 10 sprites that overlap the current scanline (ly)
                //We're mimicking that it takes 80 clks to do this
                if self.clk_ticks == 80 {
                    self.visible_sprites.clear();   //Making sure we don't keep sprites from the previous scanline
                    let mut num_of_sprite_in_scanline = 0;
                    for sprite in self.oam {
                        
                        //Checking if the sprite is in the scanline and if its visible
                        match self.is_sprite_in_scanline(&sprite) {
                            SpriteScanlineVisibility::NotInScanLine => (),
                            SpriteScanlineVisibility::NotVisible => num_of_sprite_in_scanline += 1,
                            SpriteScanlineVisibility::Visible => {
                                num_of_sprite_in_scanline += 1;
                                self.visible_sprites.push(sprite);
                            },
                        }
                        
                        if num_of_sprite_in_scanline == 10 {
                            break;
                        }
                    }
                    self.clk_ticks = 0;
                    self.ppu_registers.set_mode(PpuMode::DrawingPixels);
                    self.ppu_registers.x_scanline_coord = 0;
                }
            },
            PpuMode::DrawingPixels => { //Mode 3
                //Set initial values when starting a draw
                if self.clk_ticks == 1 {
                    self.sprite_fifo.clear();
                    self.bg_window_fifo.clear();
                    self.pixel_fetcher.drawing_window = self.pixel_fetcher.is_inside_window(&self.ppu_registers);
                    self.pixel_fetcher.x_coordinate = 0;
                    self.pixel_fetcher.win_x_coordinate = 0;
                    self.initial_pixel_shift = self.ppu_registers.scx % 8;
                    self.penalty = 12;
                }

                if self.penalty > 0 {
                    self.penalty -= 1;
                } else {
                    //Clearing fifo if were doing a transition from bg to win or vice versa
                    if self.pixel_fetcher.bg_or_win_transition(&self.ppu_registers) {
                        self.bg_window_fifo.clear();
                        if self.pixel_fetcher.is_bg_to_win(&self.ppu_registers) {
                            self.penalty += 5; //The penalty is 6 but we already started the clk tick, so this will current clk tick counts towards the penalty
                            self.stat_interrupt_req = self.raise_interrupt();
                            self.pixel_fetcher.early_transition(&self.ppu_registers);
                            return None;
                        }
                    }

                    //Fetch more bg/win tiles if the fifo is half or less full
                    while self.bg_window_fifo.len() <= 8 {
                        //Determine the tile map
                        let tile_map = match self.pixel_fetcher.determine_tile_map(&self.ppu_registers) {
                            enums::TileMapArea::_9800_9BFF => &self.tile_map_0,
                            enums::TileMapArea::_9C00_9FFF => &self.tile_map_1,
                        };
                        //Determine the tile_data_maps
                        let tile_data_map = match self.ppu_registers.lcdc.bg_win_tile_data_area {
                            TileDataArea::_8000_8FFF => (&self.tile_data_0, &self.tile_data_1),
                            TileDataArea::_8800_97FF => (&self.tile_data_2, &self.tile_data_1),
                        };
                        //Get the Data
                        let mut fetched_pixel_row = self.pixel_fetcher.fetch_pixel_row(&self.ppu_registers, 
                                                                                        tile_map, 
                                                                                        tile_data_map.0, 
                                                                                        tile_data_map.1);
                        self.bg_window_fifo.append(&mut fetched_pixel_row);
                    }

                    //Check to see if we need to render actual sprites or fill with just transparent ones
                    match self.ppu_registers.lcdc.sprite_enable {
                        State::Off => {
                            while self.sprite_fifo.len() < 8 {
                                self.sprite_fifo.push(Pixel::new_translucent_sprite_pixel());
                            }
                        },
                        State::On => {
                            if let Some(sprite) = self.visible_sprites.iter().find(|s| s.x_pos == self.ppu_registers.x_scanline_coord + 8) {
                                let mut fetched_pixel_row = self.pixel_fetcher.fetch_sprite_pixel_row(&self.ppu_registers, 
                                                                                                        &self.tile_data_0, 
                                                                                                        &self.tile_data_1, 
                                                                                                        sprite);
                                self.sprite_fifo.append(&mut fetched_pixel_row);
                            } else {
                                while self.sprite_fifo.len() < 8 {
                                    self.sprite_fifo.push(Pixel::new_translucent_sprite_pixel());
                                }
                            }
                        }
                    }

                    //Mixing sprite pixels with bg pixels
                    for pixel_idx in (0..8).rev() {
                        let sprite_pixel = self.sprite_fifo.remove(pixel_idx);
                        let pixel = self.bg_window_fifo[pixel_idx];

                        //Checking if were comparing against a bg pixel. We skip if its a sprite.
                        if !pixel.is_sprite {
                            match sprite_pixel.bg_priority.unwrap() {
                                SpritePriority::UnderBg => {
                                    if pixel.color_id == LOWEST_PRIORITY_BG_COLOR && sprite_pixel.color_id != TRANSPARENT {
                                        self.bg_window_fifo[pixel_idx] = sprite_pixel;
                                    }
                                },
                                SpritePriority::OverBg => {
                                    if sprite_pixel.color_id != TRANSPARENT {
                                        self.bg_window_fifo[pixel_idx] = sprite_pixel;
                                    }
                                },
                            }
                        }
                    }

                    //Pushing the pixel that is to be rendered
                    let pixel_to_render = self.bg_window_fifo.remove(0);
                    let final_pixel_color = match self.ppu_registers.lcdc.bg_win_priority {
                        State::On => {
                            if !pixel_to_render.is_sprite {
                                self.ppu_registers.bgp.convert_colorid_to_color(pixel_to_render.color_id)
                            } else {
                                match pixel_to_render.palette.unwrap() {    //I know it should be a sprite so just unwrap it
                                    enums::SpritePalette::Obp0 => self.ppu_registers.obp0.convert_colorid_to_color(pixel_to_render.color_id),
                                    enums::SpritePalette::Obp1 => self.ppu_registers.obp1.convert_colorid_to_color(pixel_to_render.color_id),
                                    _ => panic!("You haven't implemented CGB palette yet!")
                                }
                            }
                        },
                        State::Off => {
                            if !pixel_to_render.is_sprite {
                                PaletteColors::White
                            } else {
                                match pixel_to_render.palette.unwrap() {    //I know it should be a sprite so just unwrap it
                                    enums::SpritePalette::Obp0 => self.ppu_registers.obp0.convert_colorid_to_color(pixel_to_render.color_id),
                                    enums::SpritePalette::Obp1 => self.ppu_registers.obp1.convert_colorid_to_color(pixel_to_render.color_id),
                                    _ => panic!("You haven't implemented CGB palette yet!")
                                }
                            }
                        },
                    };

                    //Adjusting for initial pixel shifting
                    let mut pixel = Some(final_pixel_color);
                    if self.initial_pixel_shift > 0 {
                        self.initial_pixel_shift -= 1;
                        if self.pixel_fetcher.drawing_window {
                            self.ppu_registers.x_scanline_coord += 1;
                        } else {
                            pixel = None;
                        }
                    } else {
                        self.ppu_registers.x_scanline_coord += 1;
                    }

                    if self.ppu_registers.x_scanline_coord == 160 {
                        self.ppu_registers.x_scanline_coord = 0;
                        self.ppu_registers.set_mode(PpuMode::Hblank);
                    }

                    self.stat_interrupt_req = self.raise_interrupt();
                    return pixel;
                }
            },
            PpuMode::Hblank =>  {   //mode 0
                if self.clk_ticks == MAX_DRAWING_AND_HBLANK_TICKS {
                    self.clk_ticks = 0;
                    self.ppu_registers.inc_ly_reg();

                    if self.ppu_registers.ly == 144 {
                        self.ppu_registers.set_mode(PpuMode::Vblank);
                        self.vblank_interrupt_req = true;
                    } else {
                        self.ppu_registers.set_mode(PpuMode::OamScan);
                    }
                }
            },
            PpuMode::Vblank => { //Mode 1
                if self.clk_ticks == MAX_SCANLINE_CLK_TICKS {
                    self.clk_ticks = 0;
                    self.ppu_registers.inc_ly_reg();
                    if self.ppu_registers.ly > MAX_LY_VALUE {
                        self.ppu_registers.ly = 0;
                        self.ppu_registers.compare_lyc_and_ly_reg();
                        self.ppu_registers.set_mode(PpuMode::OamScan);
                    }
                }
            }
        }

        self.stat_interrupt_req = self.raise_interrupt();
        return None;
    }

    /**
     * Returns true if the ppu is active. Will only return false if the lcdc bit is turned off
     */
    pub fn is_active(&self) -> bool {
        match self.ppu_registers.lcdc.lcd_ppu_enable {
            State::On => true,
            State::Off => false,
        }
    }

    pub fn activate_ppu(&mut self) {
        self.ppu_registers.lcdc.lcd_ppu_enable = State::On;
    }

    /**
     * Returns if we should raise an interrupt or not
     */
    fn raise_interrupt(&self) -> bool {
        let int_enable_mask: u8 = self.ppu_registers.stat.read_reg_raw() >> 3;
        let mut interrupts_mask: u8 = 0;

        interrupts_mask |= match self.ppu_registers.stat.lyc_ly_compare {
            State::On => 1 << 3,
            State::Off => 0,
        };
        interrupts_mask |= match self.current_mode() {
            PpuMode::OamScan => 1 << 2,
            PpuMode::Vblank => 1 << 1,
            PpuMode::Hblank => 1 << 0,
            PpuMode::DrawingPixels => 0,
        };

        if (int_enable_mask & interrupts_mask) > 0 {
            return true;
        }
        return false;
    }

    /**
     * Returns whether the sprite is visible in the current scanline.
     * This will return false for sprites w/ x position (0 or > 168), even if 
     * they overlap the current scanline
     */
    fn is_sprite_in_scanline(&self, sprite: &Sprite) -> SpriteScanlineVisibility {
        let current_scanline = self.ppu_registers.ly + 16;
        
        if sprite.y_pos >= 160 {
            return SpriteScanlineVisibility::NotInScanLine;
        }

        let sprite_y_pos_end = match self.ppu_registers.sprite_size() {
            SpriteSize::_8x8 => sprite.y_pos + 8,
            SpriteSize::_8x16 => sprite.y_pos + 16,
        };

        //Checking if the sprite is in the scanline and if its also visible
        if current_scanline >= sprite.y_pos && current_scanline < sprite_y_pos_end {
            if sprite.x_pos == 0 || sprite.x_pos >= 168 {
                return SpriteScanlineVisibility::NotVisible;
            }
            return SpriteScanlineVisibility::Visible;
        }
        return SpriteScanlineVisibility::NotInScanLine;
    }

    pub fn current_mode(&self) -> PpuMode {
        return self.ppu_registers.stat.ppu_mode;
    }

    /**
     * Since we have structs that make accessing certain aspects of the tile 
     * easier we have to do all this conversion to get the tile we need. May 
     * need to change back to using raw arrays for bare metal implementation
     */
    pub fn read_tile_data_0(&self, address: u16) -> u8 {
        let tile_idx = (address - TILE_DATA_0_START) / 16;                  //Gives you a value between 0 and 127 to index for a tile
        let byte_idx = (address - TILE_DATA_0_START) - (tile_idx * 16);     //Gives you a value between 0 and 15 to find what byte of the tile you're looking at
        let tile_row_idx = byte_idx / 2;                                    //Gives a me a value  0 - 7 which will help tell you the row of the tile you'll need

        return match byte_idx % 2 {
            0 => self.tile_data_0[tile_idx as usize].pixel_rows[tile_row_idx as usize].lower_bits,
            _ => self.tile_data_0[tile_idx as usize].pixel_rows[tile_row_idx as usize].upper_bits,
        }
    }

    pub fn write_tile_data_0(&mut self, address: u16, value: u8) {
        let tile_idx = (address - TILE_DATA_0_START) / 16;
        let byte_idx = (address - TILE_DATA_0_START) - (tile_idx * 16);
        let tile_row_idx = byte_idx / 2;
        let lower_or_upper_byte = byte_idx % 2;

        match lower_or_upper_byte {
            0 => self.tile_data_0[tile_idx as usize].pixel_rows[tile_row_idx as usize].lower_bits = value,
            1 => self.tile_data_0[tile_idx as usize].pixel_rows[tile_row_idx as usize].upper_bits = value,
            _ => panic!("This isn't possible bro"),
        }
    }

    pub fn read_tile_data_1(&self, address: u16) -> u8 {
        let tile_idx = (address - TILE_DATA_1_START) / 16;                  //Gives you a value between 0 and 127 to index for a tile
        let byte_idx = (address - TILE_DATA_1_START) - (tile_idx * 16);     //Gives you a value between 0 and 15 to find what byte of the tile you're looking at
        let tile_row_idx = byte_idx / 2;                                    //Gives a me a value  0 - 7 which will help tell you the row of the tile you'll need

        return match byte_idx % 2 {
            0 => self.tile_data_1[tile_idx as usize].pixel_rows[tile_row_idx as usize].lower_bits,
            _ => self.tile_data_1[tile_idx as usize].pixel_rows[tile_row_idx as usize].upper_bits,
        }
    }

    pub fn write_tile_data_1(&mut self, address: u16, value: u8) {
        let tile_idx = (address - TILE_DATA_1_START) / 16;
        let byte_idx = (address - TILE_DATA_1_START) - (tile_idx * 16);
        let tile_row_idx = byte_idx / 2;

        match byte_idx % 2 {
            0 => self.tile_data_1[tile_idx as usize].pixel_rows[tile_row_idx as usize].lower_bits = value,
            _ => self.tile_data_1[tile_idx as usize].pixel_rows[tile_row_idx as usize].upper_bits = value,
        }
    }

    pub fn read_tile_data_2(&self, address: u16) -> u8 {
        let tile_idx = (address - TILE_DATA_2_START) / 16;                  //Gives you a value between 0 and 127 to index for a tile
        let byte_idx = (address - TILE_DATA_2_START) - (tile_idx * 16);     //Gives you a value between 0 and 15 to find what byte of the tile you're looking at
        let tile_row_idx = byte_idx / 2;                                    //Gives a me a value  0 - 7 which will help tell you the row of the tile you'll need

        return match byte_idx % 2 {
            0 => self.tile_data_2[tile_idx as usize].pixel_rows[tile_row_idx as usize].lower_bits,
            _ => self.tile_data_2[tile_idx as usize].pixel_rows[tile_row_idx as usize].upper_bits,
        }
    }

    pub fn write_tile_data_2(&mut self, address: u16, value: u8) {
        let tile_idx = (address - TILE_DATA_2_START) / 16;
        let byte_idx = (address - TILE_DATA_2_START) - (tile_idx * 16);
        let tile_row_idx = byte_idx / 2;

        match byte_idx % 2 {
            0 => self.tile_data_2[tile_idx as usize].pixel_rows[tile_row_idx as usize].lower_bits = value,
            _ => self.tile_data_2[tile_idx as usize].pixel_rows[tile_row_idx as usize].upper_bits = value,
        }
    }

    pub fn read_tile_map_0(&self, address: u16) -> u8 {
        return self.tile_map_0[(address - TILE_MAP_0_START) as usize];
    }

    pub fn write_tile_map_0(&mut self, address: u16, value: u8) {
        self.tile_map_0[(address - TILE_MAP_0_START) as usize] = value;
    }

    pub fn read_tile_map_1(&self, address: u16) -> u8 {
        return self.tile_map_1[(address - TILE_MAP_1_START) as usize];
    }

    pub fn write_tile_map_1(&mut self, address: u16, value: u8) {
        self.tile_map_1[(address - TILE_MAP_1_START) as usize] = value;
    }

    pub fn read_oam(&self, address: u16) -> u8 {
        let sprite_idx = (address - OAM_START) / 4;                 //Number between 0 and 39 tells the index of the sprite
        let byte_idx = (address - OAM_START) - (sprite_idx * 4);    //Number between 0 and 3, tells what byte/section of the sprite

        match byte_idx {
            0 => self.oam[sprite_idx as usize].y_pos,
            1 => self.oam[sprite_idx as usize].x_pos,
            2 => self.oam[sprite_idx as usize].tile_index,
            3 => self.oam[sprite_idx as usize].attribute_flags_raw(),   //Note this will always make the first 3 bits 0
            _ => panic!("While reading OAM ram it looks like your idx was more than 3")
        }
    }

    pub fn write_oam(&mut self, address: u16, value: u8) {
        let sprite_idx = (address - OAM_START) / 4;
        let byte_idx = (address - OAM_START) - (sprite_idx * 4);
        match byte_idx {
            0 => self.oam[sprite_idx as usize].y_pos = value,
            1 => self.oam[sprite_idx as usize].x_pos = value,
            2 => self.oam[sprite_idx as usize].tile_index = value,
            3 => self.oam[sprite_idx as usize].write_attribute_flags(value),
            _ => panic!("While writing OAM ram it looks like your idx was more than 3")
        }
    }

    pub fn read_bgp_reg(&self) -> u8 {
        return self.ppu_registers.bgp.read_reg_raw();
    }

    pub fn write_bgp_reg(&mut self, value: u8) {
        self.ppu_registers.bgp.write_reg_from_u8(value);
    }

    pub fn read_obp0_reg(&self) -> u8 {
        return self.ppu_registers.obp0.read_reg_raw();
    }

    pub fn write_obp0_reg(&mut self, value: u8) {
        self.ppu_registers.obp0.write_reg_from_u8(value);
    }

    pub fn read_obp1_reg(&self) -> u8 {
        return self.ppu_registers.obp1.read_reg_raw();
    }

    pub fn write_obp1_reg(&mut self, value: u8) {
        self.ppu_registers.obp1.write_reg_from_u8(value);
    }

    pub fn read_scy_reg(&self) -> u8 {
        return self.ppu_registers.scy;
    }

    pub fn write_scy_reg(&mut self, value: u8) {
        self.ppu_registers.scy = value;
    }

    pub fn read_scx_reg(&self) -> u8 {
        return self.ppu_registers.scx;
    }

    pub fn write_scx_reg(&mut self, value: u8) {
        self.ppu_registers.scx = value;
    }

    pub fn read_lcdc_reg(&self) -> u8 {
        return self.ppu_registers.lcdc.read_reg_raw();
    }

    pub fn write_lcdc_reg(&mut self, value: u8) {
        self.ppu_registers.lcdc.write_reg_raw(value);
    }

    pub fn read_ly_reg(&self) -> u8 {
        return self.ppu_registers.ly;
    }

    //this reg is read only you can't write to it
    // pub fn write_ly_reg(&mut self, value: u8) {
    //     self.ppu_registers.ly = value;
    // }

    pub fn read_lyc_reg(&self) -> u8 {
        return self.ppu_registers.lyc;
    }

    pub fn write_lyc_reg(&mut self, value: u8) {
        self.ppu_registers.lyc = value;
        self.ppu_registers.compare_lyc_and_ly_reg();
    }

    pub fn read_stat_reg(&self) -> u8 {
        return self.ppu_registers.stat.read_reg_raw();
    }

    pub fn write_stat_reg(&mut self, value: u8) {
        self.ppu_registers.stat.write_reg_from_u8(value);
    }

    pub fn read_wx_reg(&self) -> u8 {
        return self.ppu_registers.wx;
    }

    pub fn write_wx_reg(&mut self, value: u8) {
        self.ppu_registers.wx = value;
    }

    pub fn read_wy_reg(&self) -> u8 {
        return self.ppu_registers.wy;
    }

    pub fn write_wy_reg(&mut self, value: u8) {
        self.ppu_registers.wy = value;
    }
}


