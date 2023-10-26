use ndarray::Array2;
use crate::constants::*;
/**
 * Overview Notes
 *  -You cannot mess with pixels individually. 
 *  -Tiles are used which are 8x8 squares of pixels
 *  -You can't encode a color directly per say.
 *  -A palette is first chosen for that tile. Then it uses 2bits per pixel in the tile to determine the color in that palette
 *  -Has 3 layers from back to front in the order Background, Window, then Objects
 * 
 * Background
 *  -Consist of a tilemap, which is just a grid of tiles
 *  -Each tile though is a reference to the tile not the actual tile (saves space)
 *  -Can also be scrolled with hardware registers
 * 
 * Window
 *  -Fairly limited
 *  -No transparency
 *  -Always a rectangle
 *  -Only position of top left pixel can be controlled
 * 
 * Object
 *  -Made of 1 or 2 stacked tiles (8x8 or 8x16 pixels)
 *  -Can be displayed anywhere on screen
 *  -Can move independently of background
 *  -stored in OAM memory
 * 
 * VRAM Tile data
 *  -In VRAM between locations 0x8000-0x97FF
 *  -Each tile is 16 bytes
 *  -384 tiles can be stored in VRAM in total for DMG (CGB is different)\
 *  -Color are between the values 0-3
 *  -Objects having colors at 0 mean they are transparent
 *  -
 * 
 */

enum PpuState {
    OamScan,            //Mode2
    DrawingPixels,      //Mode3
    HorizontalBlank,    //Mode0
    VerticalBlank,      //Mode1 
}

pub struct Ppu {    
    tile_data_0: [u8; 0x800],   //$8000–$87FF
    tile_data_1: [u8; 0x800],   //$8800–$8FFF
    tile_data_2: [u8; 0x800],   //$9000–$97FF
    tile_map_0: [u8; 0x400],    //$9800-$9BFF
    tile_map_1: [u8; 0x400],    //$9C00-$9FFF
    oam: [u8; 0xA0],            //$FE00–$FE9F (Object Attribute Table) Sprite information table
    bgp_reg: u8,                //Background palette data
    obp0_reg: u8,               //Object palette 0 data
    obp1_reg: u8,               //Object palette 1 data               
    scy_reg: u8,                //Scrolling y register
    scx_reg: u8,                //Scrolling x register
    lcdc_reg: u8,               //LCD Control register
    ly_reg: u8,                 //LCD y coordinate register (current horizontal line which might be able to be drawn, being drawn, or just been drawn)
    lyc_reg: u8,                //LY compare register. Can use this register to trigger an interrupt when LY reg and this reg are the same value 
    stat_reg: u8,               //LCD status register
    wx_reg: u8,                 //Window x position
    wy_reg: u8,                 //Window y position
    ppu_state: PpuState,
    ppu_clk_ticks: u16
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            tile_data_0: [0; 0x800],
            tile_data_1: [0; 0x800],
            tile_data_2: [0; 0x800],
            tile_map_0: [0; 0x400],
            tile_map_1: [0; 0x400],
            oam: [0; 0xA0],
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
        }
    }

    pub fn cycle(&mut self) {
        self.ppu_clk_ticks += 1;

        match self.ppu_state {
            PpuState::OamScan => {
                if self.ppu_clk_ticks == 80 {
                    self.ppu_clk_ticks = 0;
                    self.ppu_state = PpuState::DrawingPixels;
                }
            },
            PpuState::DrawingPixels => {
                if self.ppu_clk_ticks == 172 {  //This number is not for certain this can vary
                    self.ppu_clk_ticks = 0;
                    self.ppu_state = PpuState::HorizontalBlank;
                }
                todo!("Need to implement the variable about of ticks this mode state can take");
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
        return self.tile_data_0[(address - TILE_DATA_0_START) as usize];
    }

    pub fn write_tile_data_0(&mut self, address: u16, value: u8) {
        self.tile_data_0[(address - TILE_DATA_0_START) as usize] = value;
    }

    pub fn read_tile_data_1(&self, address: u16) -> u8 {
        return self.tile_data_1[(address - TILE_DATA_1_START) as usize];
    }

    pub fn write_tile_data_1(&mut self, address: u16, value: u8) {
        self.tile_data_1[(address - TILE_DATA_1_START) as usize] = value;
    }

    pub fn read_tile_data_2(&self, address: u16) -> u8 {
        return self.tile_data_2[(address - TILE_DATA_2_START) as usize];
    }

    pub fn write_tile_data_2(&mut self, address: u16, value: u8) {
        self.tile_data_2[(address - TILE_DATA_2_START) as usize] = value;
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
        return self.oam[(address - OAM_START) as usize];
    }

    pub fn write_oam(&mut self, address: u16, value: u8) {
        self.oam[(address - OAM_START) as usize] = value;
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


