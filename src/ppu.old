use std::collections::VecDeque;
use crate::binary_utils;
use crate::constants::*;

#[derive(Default, Clone, Copy)]
struct Pixel {
    color_id: u8,
    palette: u8,
    obj_to_bg_priority: bool,
}

impl Pixel {
    /**
     * Returns a newly constructed pixel. This pixel will be a low priority/transparent pixel
     */
    fn new() -> Self {
        Self {
            color_id: 0,
            palette: 0,
            obj_to_bg_priority: false,
        }
    }

    /**
     * Tells you if the pixel is transparent. This should only be used on sprite pixels. Prob
     * Should make 2 structs with one for sprites and other for window/bg
     */
    fn is_transparent(&self) -> bool {
        if self.color_id == 0 {
            return true; 
        }
        return false;
    }
}

#[derive(PartialEq, PartialOrd)]
pub enum PpuState {
    OamScan,            //Mode2
    DrawingPixels,      //Mode3
    HorizontalBlank,    //Mode0
    VerticalBlank,      //Mode1
}

#[derive(Clone, Copy)]
struct Sprite {
    y_pos: u8,
    x_pos: u8,
    tile_index: u8,
    attribute_flags: u8,
}

impl Sprite {
    fn new() -> Self {
        Self {
            y_pos: 0,
            x_pos: 0,
            tile_index: 0,
            attribute_flags: 0,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Tile {
    pub pixel_rows: [u8; 16],   //Every 2 bytes is a pixel row
}

impl Tile {
    fn new() -> Self {
        Self { pixel_rows: [0; 16] }
    }
}

pub struct Ppu {
    tile_data_0: [Tile; 128],   //$8000–$87FF
    tile_data_1: [Tile; 128],   //$8800–$8FFF
    tile_data_2: [Tile; 128],   //$9000–$97FF
    tile_map_0: [u8; 0x400],    //$9800-$9BFF
    tile_map_1: [u8; 0x400],    //$9C00-$9FFF
    oam: [Sprite; 40],          //$FE00–$FE9F (Object Attribute Table) Sprite information table
    visible_sprites: Vec<Sprite>,
    bgp_reg: u8,                //$FF47 - Background palette data
    obp0_reg: u8,               //$FF48 - Object palette 0 data
    obp1_reg: u8,               //$FF49 - Object palette 1 data
    scy_reg: u8,                //$FF42 - Scrolling y register
    scx_reg: u8,                //$FF43 - Scrolling x register
    lcdc_reg: u8,               //$FF40 - LCD Control register
    ly_reg: u8,                 //$FF44 - LCD y coordinate register (current horizontal line which might be able to be drawn, being drawn, or just been drawn)
    lyc_reg: u8,                //$FF45 - LY compare register. Can use this register to trigger an interrupt when LY reg and this reg are the same value
    stat_reg: u8,               //$FF41 - LCD status register
    wx_reg: u8,                 //$FF4B - Window x position
    wy_reg: u8,                 //$FF4A - Window y position
    pub state: PpuState,
    clk_ticks: u16,
    drawing_penalty: u8,
    bg_window_fifo: VecDeque<Pixel>,
    sprite_fifo: VecDeque<Pixel>,
    x_scanline_coord: u8,       //This is not a real register but will help us keep track of the x position of the scanline
    drawing_window: bool,
    x_fetcher_coord: u8,
    pub vblank_interrupt_requested: bool,
    pub stat_interrupt_requested: bool,
    stat_line: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            tile_data_0: [Tile::new(); 128],
            tile_data_1: [Tile::new(); 128],
            tile_data_2: [Tile::new(); 128],
            tile_map_0: [0; 0x400],
            tile_map_1: [0; 0x400],
            visible_sprites: Vec::with_capacity(10),
            oam: [Sprite::new(); 40],
            bgp_reg: 0,
            obp0_reg: 0,
            obp1_reg: 0,
            scy_reg: 0,
            scx_reg: 0,
            lcdc_reg: 0,
            ly_reg: 0,
            lyc_reg: 0,
            stat_reg: 0,
            wx_reg: 0,
            wy_reg: 0,
            state: PpuState::OamScan,
            clk_ticks: 0,
            drawing_penalty: 0,
            bg_window_fifo: VecDeque::with_capacity(16),
            sprite_fifo: VecDeque::with_capacity(16),
            x_scanline_coord: 0,
            drawing_window: false,
            x_fetcher_coord: 0,
            vblank_interrupt_requested: false,
            stat_interrupt_requested: false,
            stat_line: false,
        }
    }

    pub fn cycle(&mut self) {
        self.clk_ticks += 1;
        self.update_lyc_and_ly_compare();

        match self.state {
            PpuState::OamScan => {
                if self.clk_ticks == 1 {
                    //Need to change the mode in the stat register
                    binary_utils::reset_bit(self.stat_reg, 0);
                    binary_utils::set_bit(self.stat_reg, 1);
                }

                if self.clk_ticks == 80 {
                    let mut num_of_sprites_found = 0;
                    self.visible_sprites.clear();

                    for sprite in self.oam {
                        if self.is_sprite_in_scanline(&sprite) {
                            num_of_sprites_found += 1;
                            if sprite.x_pos != 0 && sprite.x_pos < 168 { //Checking if not hidden
                                self.visible_sprites.push(sprite);
                            }
                        }
                        //Break when we found the 10 sprites we need
                        if num_of_sprites_found == 10 {
                            break;
                        }
                    }
                    self.clk_ticks = 0;
                    self.state = PpuState::DrawingPixels;
                }
            },
            PpuState::DrawingPixels => {
                if self.clk_ticks == 1 {
                    self.drawing_penalty += self.scx_reg % 8;
                    self.sprite_fifo.clear();
                    self.bg_window_fifo.clear();
                    self.x_fetcher_coord = 0;
                    self.x_scanline_coord = 0;
                    self.ly_reg = 0;
                    self.stat_reg |= 0x3;
                }

                //Just leave if we have a drawing penalty otherwise draw a pixel
                if self.drawing_penalty != 0 {
                    self.drawing_penalty -= 1;
                } else {
                    //Filling the bg/window fifo making sure it has more than 8 pixels at all times
                    while self.bg_window_fifo.len() <= 8 {
                        let mut new_pixel_row = self.fetch_tile_pixel_row();
                        self.bg_window_fifo.append(&mut new_pixel_row);
                    }

                    //Checking if we need to fetch an object and add it to the fifo
                    if self.visible_sprites.iter().any(|sprite| (*sprite).x_pos == self.x_scanline_coord + 8) {
                        let mut new_object_pixel_row = self.fetch_object_tile_row();
                        self.sprite_fifo.append(&mut new_object_pixel_row);
                    }

                    //Start pixel mixing
                    let pixel_to_push = {
                        let background_pixel = self.bg_window_fifo.pop_front().unwrap();
                        if !self.sprite_fifo.is_empty() {
                            let sprite_pixel = self.sprite_fifo.pop_front().unwrap();
                            if !sprite_pixel.is_transparent() && binary_utils::get_bit(self.lcdc_reg, 1) != 0 && !sprite_pixel.obj_to_bg_priority {
                                sprite_pixel
                            } else {
                                background_pixel
                            }
                        } else {
                            background_pixel
                        }
                    };
                }

                if self.clk_ticks == 172 {  //This number is not for certain this can vary
                    self.x_scanline_coord = 0;
                    self.clk_ticks = 0;
                    self.state = PpuState::HorizontalBlank;
                }                                                                        

                self.x_scanline_coord += 1;
            },
            PpuState::HorizontalBlank => {
                if self.clk_ticks == 1 {
                    binary_utils::reset_bit(self.stat_reg, 0);
                    binary_utils::reset_bit(self.stat_reg, 0);
                }


                if self.clk_ticks == 87 {
                    self.clk_ticks = 0;
                    self.state = PpuState::OamScan;
                    if self.ly_reg >= 144 {
                        self.state = PpuState::VerticalBlank;
                    }
                }
                todo!("Need to implement the variable about of ticks this mode state can take");
            },
            PpuState::VerticalBlank => {
                if self.clk_ticks == 1 {
                    binary_utils::reset_bit(self.stat_reg, 1);
                    binary_utils::set_bit(self.stat_reg, 0);
                    self.vblank_interrupt_requested = true;
                }

                if self.clk_ticks == 456 && self.ly_reg > 153 {  //Not sure if 153 is good to use
                    self.state = PpuState::OamScan;
                }

                todo!("Need to see if the ly reg value check is correct");
            },
        }

        self.update_stat_interrupt();
    }

    fn update_lyc_and_ly_compare(&mut self) {
        if self.lyc_reg == self.ly_reg {
            binary_utils::set_bit(self.stat_reg, 2);
        } else {
            binary_utils::reset_bit(self.stat_reg, 2);
        }
    }

    /**
     * This will update the stat interrupt line and request an interrupt it needed
     */
    fn update_stat_interrupt(&mut self) {
        let lyc_int = binary_utils::get_bit(self.stat_reg, 6) != 0 && binary_utils::get_bit(self.stat_reg, 2) != 0;
        let oam_scan_int = self.state == PpuState::OamScan && binary_utils::get_bit(self.stat_reg, 5) != 0;
        let hblank_int = self.state == PpuState::HorizontalBlank && binary_utils::get_bit(self.stat_reg, 0) != 0;
        let vblank_int = self.state == PpuState::VerticalBlank && binary_utils::get_bit(self.stat_reg, 1) != 0;
        let stat_line = lyc_int || oam_scan_int || hblank_int || vblank_int;

        if !self.stat_line && stat_line {
            self.stat_interrupt_requested = true;
        }

        self.stat_line = stat_line;
    }

    pub fn read_tile_data_0(&self, address: u16) -> u8 {
        let tile_idx = (address - TILE_DATA_0_START) / 16;
        let byte_idx = (address - TILE_DATA_0_START) - (tile_idx * 16);
        return self.tile_data_0[tile_idx as usize].pixel_rows[byte_idx as usize];
    }

    pub fn write_tile_data_0(&mut self, address: u16, value: u8) {
        let tile_idx = (address - TILE_DATA_0_START) / 16;
        let byte_idx = (address - TILE_DATA_0_START) - (tile_idx * 16);
        self.tile_data_0[tile_idx as usize].pixel_rows[byte_idx as usize] = value;
    }

    pub fn read_tile_data_1(&self, address: u16) -> u8 {
        let tile_idx = (address - TILE_DATA_1_START) / 16;
        let byte_idx = (address - TILE_DATA_1_START) - (tile_idx * 16);
        return self.tile_data_1[tile_idx as usize].pixel_rows[byte_idx as usize];
    }

    pub fn write_tile_data_1(&mut self, address: u16, value: u8) {
        let tile_idx = (address - TILE_DATA_1_START) / 16;
        let byte_idx = (address - TILE_DATA_1_START) - (tile_idx * 16);
        self.tile_data_1[tile_idx as usize].pixel_rows[byte_idx as usize] = value;
    }

    pub fn read_tile_data_2(&self, address: u16) -> u8 {
        let tile_idx = (address - TILE_DATA_2_START) / 16;
        let byte_idx = (address - TILE_DATA_2_START) - (tile_idx * 16);
        return self.tile_data_2[tile_idx as usize].pixel_rows[byte_idx as usize];
    }

    pub fn write_tile_data_2(&mut self, address: u16, value: u8) {
        let tile_idx = (address - TILE_DATA_2_START) / 16;
        let byte_idx = (address - TILE_DATA_2_START) - (tile_idx * 16);
        self.tile_data_2[tile_idx as usize].pixel_rows[byte_idx as usize] = value;
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
        let sprite_idx = (address - OAM_START) / 4;
        let byte_idx = (address - OAM_START) - (sprite_idx * 4);

        match byte_idx {
            0 => self.oam[sprite_idx as usize].y_pos,
            1 => self.oam[sprite_idx as usize].x_pos,
            2 => self.oam[sprite_idx as usize].tile_index,
            3 => self.oam[sprite_idx as usize].attribute_flags,
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
            3 => self.oam[sprite_idx as usize].attribute_flags = value,
            _ => panic!("While writing OAM ram it looks like your idx was more than 3")
        }
    }

    pub fn read_bgp_reg(&self) -> u8 {
        return self.bgp_reg;
    }

    pub fn write_bgp_reg(&mut self, value: u8) {
        self.bgp_reg = value;
    }

    pub fn read_obp0_reg(&self) -> u8 {
        return self.obp0_reg;
    }

    pub fn write_obp0_reg(&mut self, value: u8) {
        self.obp0_reg = value;
    }

    pub fn read_obp1_reg(&self) -> u8 {
        return self.obp1_reg;
    }

    pub fn write_obp1_reg(&mut self, value: u8) {
        self.obp1_reg = value;
    }

    pub fn read_scy_reg(&self) -> u8 {
        return self.scy_reg;
    }

    pub fn write_scy_reg(&mut self, value: u8) {
        self.scy_reg = value;
    }

    pub fn read_scx_reg(&self) -> u8 {
        return self.scx_reg;
    }

    pub fn write_scx_reg(&mut self, value: u8) {
        self.scx_reg = value;
    }

    pub fn read_lcdc_reg(&self) -> u8 {
        return self.lcdc_reg;
    }

    pub fn write_lcdc_reg(&mut self, value: u8) {
        self.lcdc_reg = value;
    }

    pub fn read_ly_reg(&self) -> u8 {
        return self.ly_reg;
    }

    pub fn write_ly_reg(&mut self, value: u8) {
        self.ly_reg = value;
    }

    pub fn read_lyc_reg(&self) -> u8 {
        return self.lyc_reg;
    }

    pub fn write_lyc_reg(&mut self, value: u8) {
        self.lyc_reg = value;
    }

    pub fn read_stat_reg(&self) -> u8 {
        return self.stat_reg;
    }

    pub fn write_stat_reg(&mut self, value: u8) {
        self.stat_reg = value;
    }

    pub fn read_wx_reg(&self) -> u8 {
        return self.wx_reg;
    }

    pub fn write_wx_reg(&mut self, value: u8) {
        self.wx_reg = value;
    }

    pub fn read_wy_reg(&self) -> u8 {
        return self.wy_reg;
    }

    pub fn write_wy_reg(&mut self, value: u8) {
        self.wy_reg = value;
    }

    /*
        This will return a reference to either either the tile map starting at $9800 or $9C00
     */
    fn determine_tile_map(&self) -> &[u8; 0x400] {
        if (binary_utils::get_bit(self.lcdc_reg, 6) != 0 &&
            self.x_scanline_coord + 7 >= self.wx_reg && self.ly_reg >= self.wy_reg) ||
                binary_utils::get_bit(self.lcdc_reg, 3) != 0 &&
                    self.x_scanline_coord + 7 < self.wx_reg && self.ly_reg < self.wy_reg {
            return &self.tile_map_1;
        }
        return &self.tile_map_0;
    }

    /*
    Obviously tells you if the sprite is on the given scanline. BUT NOTE this will also include
    sprites that are on x = 0 or x >= 168, which will have them not be visible on the screen.
    This means they could still count towards the 10 object per scanline limit
     */
    fn is_sprite_in_scanline(&self, sprite: &Sprite) -> bool {
        let sprite_y_pos_end = {
            if binary_utils::get_bit(self.lcdc_reg, 2) == 0 {
                sprite.y_pos + 8
            } else {
                sprite.y_pos + 16
            }
        };
        if self.ly_reg + 16 >= sprite.y_pos && self.ly_reg + 16 < sprite_y_pos_end {
            return true;
        }
        return false;
    }

    /**
     * Will be called when we need to fetch the next pixel row
     */
    fn fetch_tile_pixel_row(&mut self) -> VecDeque<Pixel> {
        //Checking if were transitioning from bg to window drawing or the vice versa
        if self.x_scanline_coord + 7 >= self.wx_reg && self.ly_reg >= self.wy_reg && !self.drawing_window {
            self.drawing_window = true;
            self.bg_window_fifo.clear();
            self.x_fetcher_coord = self.x_scanline_coord / 8;
            todo!("Need to implement the when wx == 0 and SCX & 7 > 0. To shorten the mode 3 by 1 dot. And potentially the window bug");
        } else if self.x_scanline_coord + 7 < self.wx_reg && self.ly_reg < self.wy_reg && self.drawing_window {
            self.drawing_window = false;
            self.bg_window_fifo.clear();
            self.x_fetcher_coord = self.x_scanline_coord / 8;
        }

        let tile_map = self.determine_tile_map();

        //Finding which tile in the tile map to choose from and getting tile data offset ($0-$FF)
        let tile_map_x_coord = ((self.scx_reg / 8) + self.x_fetcher_coord) & 0x1F;  //overflow error huh future brandon or need to not worry about the last 3 bits of scx reg
        let tile_map_y_coord = (self.scy_reg + self.ly_reg) & 255;  //overflow here to huh. Shouldn't listen to yourself in the beginning
        let tile_idx = tile_map_x_coord + ((tile_map_y_coord / 8) * 32);
        let tile_data_idx = tile_map[tile_idx as usize];

        //Getting the actual tile data
        let is_8000_addressing = binary_utils::get_bit(self.lcdc_reg, 4) != 0;
        let tile = match tile_data_idx {
            0 ..= 127 if is_8000_addressing => &self.tile_data_0[tile_data_idx as usize],
            128 ..= 255 if is_8000_addressing => &self.tile_data_1[(tile_data_idx - 128) as usize],
            0 ..= 127 => &self.tile_data_2[tile_data_idx as usize],
            128 ..= 255 => &self.tile_data_1[(tile_data_idx - 128) as usize],
        };

        //Getting the 2 bytes to build the 8 pixels
        let row_idx = (self.ly_reg - ((self.ly_reg / 8) * 8)) * 2;
        let pixel_row_lsb = tile.pixel_rows[row_idx as usize];       
        let pixel_row_msb = tile.pixel_rows[(row_idx+1) as usize];   

        //Building pixels and putting them in the queue to be pushed to the fifo
        let mut pixel_row: VecDeque<Pixel> = vec![].into();
        for bit_pos in (0..8).rev() {
            let bit_0 = binary_utils::get_bit(pixel_row_lsb, bit_pos);
            let bit_1 = binary_utils::get_bit(pixel_row_msb, bit_pos);
            let bit = (bit_1 << 1) | bit_0;

            let pixel = Pixel {
                color_id: bit,
                palette: 0,
                obj_to_bg_priority: false,
            };

            pixel_row.push_back(pixel);
        }

        self.x_fetcher_coord += 1;
        return pixel_row;
    }

    /**
     * Currently this function should only really be called if you have already checked that you have a sprite to
     * draw
     */
    fn fetch_object_tile_row(&mut self) -> VecDeque<Pixel> {
        //Detemine what sprite to get
        let sprite = self.visible_sprites.iter().find(|sprite| (*sprite).x_pos == self.x_scanline_coord + 8).unwrap();

        let object_tile = match sprite.tile_index {
            0 ..= 127 => &self.tile_data_0[sprite.tile_index as usize],
            128 ..= 255 => &self.tile_data_1[(sprite.tile_index - 128) as usize],
        };

        //Getting the 2 bytes to build the 8 pixels
        let row_idx = (self.ly_reg - ((self.ly_reg / 8) * 8)) * 2;
        let pixel_row_lsb = object_tile.pixel_rows[row_idx as usize];       
        let pixel_row_msb = object_tile.pixel_rows[(row_idx+1) as usize];   

        //Building pixels and putting them in the queue to be pushed to the fifo
        let mut pixel_row: VecDeque<Pixel> = vec![].into();
        for bit_pos in (0..8).rev() {
            let bit_0 = binary_utils::get_bit(pixel_row_lsb, bit_pos);
            let bit_1 = binary_utils::get_bit(pixel_row_msb, bit_pos);
            let bit = (bit_1 << 1) | bit_0;

            let pixel = Pixel {
                color_id: bit,
                palette: binary_utils::get_bit(sprite.attribute_flags, 4),
                obj_to_bg_priority: binary_utils::get_bit(sprite.attribute_flags, 7) != 0,
            };

            pixel_row.push_back(pixel);
        }

        return pixel_row;
    }
}


