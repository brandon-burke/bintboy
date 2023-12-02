pub mod enums;
mod registers;
mod tile_and_sprite;

use self::registers::PpuRegisters;
use self::enums::{PpuMode, SpriteSize};
use self::tile_and_sprite::*;
use crate::gameboy::constants::*;
pub struct Ppu {
    tile_data_0: [Tile; 128],       //$8000–$87FF
    tile_data_1: [Tile; 128],       //$8800–$8FFF
    tile_data_2: [Tile; 128],       //$9000–$97FF
    tile_map_0: [u8; 0x400],        //$9800-$9BFF
    tile_map_1: [u8; 0x400],        //$9C00-$9FFF
    oam: [Sprite; 40],              //$FE00–$FE9F (Object Attribute Table) Sprite information table
    ppu_registers: PpuRegisters,    //Houses all ppu registers
    clk_ticks: u16,                 //How many cpu ticks have gone by
    visible_sprites: Vec<Sprite>,   //Visible Sprites on current scanline
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
        }
    }

    pub fn cycle(&mut self) {
        self.clk_ticks += 1;    //Keeps track of how many ticks during a mode

        match self.current_mode() {
            PpuMode::OamScan => {
                //Finding up to 10 sprites that overlap the current scanline (ly)
                //We're mimicking that it takes 80 clks to do this
                if self.clk_ticks == 80 {
                    self.visible_sprites.clear();   //Making sure we don't keep sprites from the previous scanline
                    for sprite in self.oam {
                        if self.is_sprite_visible_in_scanline(&sprite) {
                            self.visible_sprites.push(sprite);
                        }
    
                        if self.visible_sprites.len() == 10 {
                            break;
                        }
                    }
                    self.clk_ticks = 0;
                    self.ppu_registers.set_mode(PpuMode::DrawingPixels);
                }
            },
            PpuMode::DrawingPixels => todo!(),
            PpuMode::Hblank => todo!(),
            PpuMode::Vblank => todo!(),
        }
    }

    /**
     * Returns whether the sprite is visible in the current scanline.
     * This will return false for sprites that overlap the current scanline, 
     * but their x position (0 or >168) makes them not visible
     */
    fn is_sprite_visible_in_scanline(&self, sprite: &Sprite) -> bool {
        let current_scanline = self.ppu_registers.ly + 16;
        let sprite_y_pos_end = match self.ppu_registers.sprite_size() {
            SpriteSize::_8x8 => sprite.y_pos + 8,
            SpriteSize::_8x16 => sprite.y_pos + 16,
        };

        //Checking is the sprite is visible
        if current_scanline >= sprite.y_pos && 
            current_scanline < sprite_y_pos_end &&
            sprite.x_pos != 0 &&
            sprite.x_pos < 168 {
            return true;
        }
        return false;
    }

    pub fn current_mode(&self) -> PpuMode {
        return self.ppu_registers.stat.ppu_mode;
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
        return self.ppu_registers.bgp;
    }

    pub fn write_bgp_reg(&mut self, value: u8) {
        self.ppu_registers.bgp = value;
    }

    pub fn read_obp0_reg(&self) -> u8 {
        return self.ppu_registers.obp0_reg;
    }

    pub fn write_obp0_reg(&mut self, value: u8) {
        self.ppu_registers.obp0_reg = value;
    }

    pub fn read_obp1_reg(&self) -> u8 {
        return self.ppu_registers.obp1_reg;
    }

    pub fn write_obp1_reg(&mut self, value: u8) {
        self.ppu_registers.obp1_reg = value;
    }

    pub fn read_scy_reg(&self) -> u8 {
        return self.ppu_registers.scy_reg;
    }

    pub fn write_scy_reg(&mut self, value: u8) {
        self.ppu_registers.scy_reg = value;
    }

    pub fn read_scx_reg(&self) -> u8 {
        return self.ppu_registers.scx_reg;
    }

    pub fn write_scx_reg(&mut self, value: u8) {
        self.ppu_registers.scx_reg = value;
    }

    pub fn read_lcdc_reg(&self) -> u8 {
        return self.ppu_registers.lcdc_reg;
    }

    pub fn write_lcdc_reg(&mut self, value: u8) {
        self.ppu_registers.lcdc_reg = value;
    }

    pub fn read_ly_reg(&self) -> u8 {
        return self.ppu_registers.ly_reg;
    }

    pub fn write_ly_reg(&mut self, value: u8) {
        self.ppu_registers.ly_reg = value;
    }

    pub fn read_lyc_reg(&self) -> u8 {
        return self.ppu_registers.lyc_reg;
    }

    pub fn write_lyc_reg(&mut self, value: u8) {
        self.ppu_registers.lyc_reg = value;
    }

    pub fn read_stat_reg(&self) -> u8 {
        return self.ppu_registers.stat_reg;
    }

    pub fn write_stat_reg(&mut self, value: u8) {
        self.ppu_registers.stat_reg = value;
    }

    pub fn read_wx_reg(&self) -> u8 {
        return self.ppu_registers.wx_reg;
    }

    pub fn write_wx_reg(&mut self, value: u8) {
        self.ppu_registers.wx_reg = value;
    }

    pub fn read_wy_reg(&self) -> u8 {
        return self.ppu_registers.wy_reg;
    }

    pub fn write_wy_reg(&mut self, value: u8) {
        self.ppu_registers.wy_reg = value;
    }
}


/**
 * Represents the pixel fetcher in the gameboy. It'll house all the things 
 * necessary to fetch sprite, bg, and window pixels
 */
struct PixelFetcher {

}

