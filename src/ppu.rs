use ndarray::Array2;
use crate::binary_utils;
use crate::constants::*;

enum PpuState {
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

#[derive(Default, Clone, Copy)]
struct Pixel {
    color_id: u8,
    palette: u8,
    is_background: bool,
}

impl Pixel {
    fn new() -> Self {
        Self {
            color_id: 0,
            palette: 0,
            is_background: false,
        }
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
    ppu_state: PpuState,
    ppu_clk_ticks: u16,
    view_port: Array2<Tile>,
    drawing_penalty: u8,
    bg_window_fifo: [Pixel; 16],
    sprite_fifo: [Pixel; 16],
    x_scanline_coord: u8,       //This is not a real register but will help us keep track of the x position of the scanline
    queued_pixel: Pixel,
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
            ppu_state: PpuState::OamScan,
            ppu_clk_ticks: 0,
            view_port: Array2::default((160, 144)),
            drawing_penalty: 0,
            bg_window_fifo: [Pixel::new(); 16],
            sprite_fifo: [Pixel::new(); 16],
            x_scanline_coord: 0,
            queued_pixel: Pixel::new(),
        }
    }

    pub fn cycle(&mut self) {
        self.ppu_clk_ticks += 1;

        match self.ppu_state {
            PpuState::OamScan => {
                if self.ppu_clk_ticks == 80 {
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
                    self.ppu_clk_ticks = 0;
                    self.ppu_state = PpuState::DrawingPixels;
                }
            },
            PpuState::DrawingPixels => {
                //Determine which tilemap we are using
                let tile_map = self.determine_tile_map();

                //Figure out which tile idx to get in the tile map (find through scx and scy regs)
                //(SCX/8) tells us where the first tile is since scx scrolls the viewport
                let tile_map_x_coord = (self.scx_reg + self.x_scanline_coord) / 8;  //overflow error huh future brandon
                let tile_map_y_coord = (self.scy_reg + self.ly_reg) / 8;
                let tile_idx = tile_map_x_coord + (tile_map_y_coord * 32);

                //Get the tile data idx which will tell us where in VRAM to get the tile from
                //Remember this is not a full address but rather an offset
                let tile_data_idx = tile_map[tile_idx as usize];

                //Checking which tilemap to use this is dependent on bit 4 of the lcdc reg and if
                //The tile is a object or not
                let is_8000_addressing = binary_utils::get_bit(self.lcdc_reg, 4) != 0;
                let tile = match tile_data_idx {
                    0 ..= 127 if is_8000_addressing => &self.tile_data_0[tile_data_idx as usize],
                    128 ..= 255 if is_8000_addressing => &self.tile_data_1[(tile_data_idx - 128) as usize],
                    0 ..= 127 => &self.tile_data_2[tile_data_idx as usize],
                    128 ..= 255 => &self.tile_data_1[(tile_data_idx - 128) as usize],
                    _ => panic!("Tile data map idx issue"),
                };

                let row_idx = (self.ly_reg - ((self.ly_reg / 8) * 8)) * 2;
                let tile_pixel_row_1 = tile.pixel_rows[row_idx as usize];   //least significant byte row
                let tile_pixel_row_2 = tile.pixel_rows[(row_idx+1) as usize];   //most significant byte row

                let fake_pixel_fifo: Vec<Pixel> = vec![];
                //Build all the pixels in the row
                for _ in 0..8 {
                    let bit_0 = binary_utils::get_bit(tile_pixel_row_1, 7);
                    let bit_1 = binary_utils::get_bit(tile_pixel_row_2, 7);
                    let bit = (bit_1 << 1) | bit_0;

                    let pixel = Pixel {
                        color_id: bit,
                        palette: 0,
                        is_background: true,
                    };
                }




                if self.ppu_clk_ticks == 172 {  //This number is not for certain this can vary
                    self.x_scanline_coord = 0;
                    self.ppu_clk_ticks = 0;
                    self.ppu_state = PpuState::HorizontalBlank;
                }

                self.x_scanline_coord += 1;
                todo!("Need to implement the variable about of ticks this mode state can take (the penalty)");
            },
            PpuState::HorizontalBlank => {
                if self.ppu_clk_ticks == 87 {
                    self.ppu_clk_ticks = 0;
                    self.ppu_state = PpuState::OamScan;
                    if self.ly_reg >= 144 {
                        self.ppu_state = PpuState::VerticalBlank;
                    }
                }
                todo!("Need to implement the variable about of ticks this mode state can take");
            },
            PpuState::VerticalBlank => {
                if self.ppu_clk_ticks == 456 && self.ly_reg > 153 {  //Not sure if 153 is good to use
                    self.ppu_state = PpuState::OamScan;
                }

                todo!("Need to see if the ly reg value check is correct");
            },
        }
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
     * Will shift the entire frame of the view port to the right
     */
    fn scroll_frame_right(grid: &mut Array2<u8>) {
        for row in grid.rows_mut() {
            let mut prev_ele = *row.last().unwrap();
            for ele in row {
                let temp = *ele;
                *ele = prev_ele;
                prev_ele = temp;
            }
        }
    }

    /**
     * Will shift the entire frame of the view port to the left
     */
    fn scroll_left(grid: &mut Array2<u8>) {
        for mut row in grid.rows_mut() {
            let mut prev_ele = *row.first().unwrap();
            for ele in row.iter_mut().rev() {
                let temp = *ele;
                *ele = prev_ele;
                prev_ele = temp;
            }
        }
    }
}


