


struct Ppu {
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
    fn new() -> Self {
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

    fn cycle(&mut self) {
        self.clk_ticks += 1;    //Keeps track of how many ticks during a mode

        match self.ppu_registers.stat.ppu_mode {
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
                    self.ppu_registers.stat.ppu_mode = PpuMode::DrawingPixels;
                }
            },
            PpuMode::DrawingPixels => {
                
            },
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
        let sprite_y_pos_end = match self.ppu_registers.lcdc.sprite_size {
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
}


/*
    Represents a 8x8 square of pixels. Here we have an array of PixelRows
    where each PixelRow in the array represents the data of a row of pixels
    in the Tile
 */
#[derive(Clone, Copy)]
struct Tile {
    pixel_rows: [TileRow; 8]
}

impl Tile {
    fn new() -> Self {
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
struct Sprite {
    y_pos: u8,
    x_pos: u8,
    tile_index: u8,
    priority: SpritePriority,
    y_flip: Orientation,
    x_flip: Orientation,
    dmg_palette: SpritePalette,     //Non CGB Mode only
    bank: VramBank,                 //CGB Mode only
    cgb_palette: SpritePalette      //CGB Mode only
}

impl Sprite {
    fn new() -> Self {
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

#[derive(Clone, Copy)]
enum SpritePriority{
    OverBg,     //Sprite should draw over background and window
    UnderBg,    //Background and window colors 1-3 are drawn over the Sprite
}


/* Represents whether a sprite is mirrored or not */
#[derive(Clone, Copy)]
enum Orientation {
    Normal,
    Mirrored,
}

/* Represents the options for a palette that a sprite can use */
#[derive(Clone, Copy)]
enum SpritePalette {
    Obp0,
    Obp1,
    Obp2,
    Obp3,
    Obp4,
    Obp5,
    Obp6,
    Obp7,
}

#[derive(Clone, Copy)]
enum VramBank {
    Bank0,
    Bank1,
}

/* Represents the LCD Control register (LCDC) */
struct LcdcReg {
    lcd_ppu_enable: State,
    win_tile_map_area: TileMapArea,
    win_enable: State,
    bg_win_tile_data_area: TileDataArea,
    bg_tile_map_area: TileMapArea,
    sprite_size: SpriteSize,
    sprite_enable: State,
    bg_win_priority: State,
}

impl LcdcReg {
    fn new() -> Self {
        Self {
            lcd_ppu_enable: State::Off,
            win_tile_map_area: TileMapArea::_9800,
            win_enable: State::Off,
            bg_win_tile_data_area: TileDataArea::_8000,
            bg_tile_map_area: TileMapArea::_9800,
            sprite_size: SpriteSize::_8x8,
            sprite_enable: State::Off,
            bg_win_priority: State::Off,
        }
    }
}

struct StatReg {
    unused_bit_7: u8,
    lyc_int_select: State,
    mode_2_int_select: State,
    mode_1_int_select: State,
    mode_0_int_select: State,
    lyc_ly_compare: State,      //Read-Only
    ppu_mode: PpuMode,          //Read-Only
}

impl StatReg {
    fn new() -> Self {
        Self {
            unused_bit_7: 0,
            lyc_int_select: State::Off,
            mode_2_int_select: State::Off,
            mode_1_int_select: State::Off,
            mode_0_int_select: State::Off,
            lyc_ly_compare: State::Off,
            ppu_mode: PpuMode::OamScan,
        }
    }
}

enum State {
    On,
    Off,
}

enum TileMapArea {
    _9800,
    _9C00,
}

enum TileDataArea {
    _8800,
    _8000,
}

enum SpriteSize {
    _8x8,
    _8x16,
}

enum PpuMode {
    OamScan,        //Mode 2
    DrawingPixels,  //Mode 3
    Hblank,         //Mode 0
    Vblank,         //Mode 1
}

struct PpuRegisters {
    lcdc: LcdcReg,      //$FF40 - LCD Control register
    ly: u8,             //READ-ONLY -> $FF44 - LCD y coordinate register (current horizontal line which might be able to be drawn, being drawn, or just been drawn)
    lyc: u8,            //$FF45 - LY compare register. Can use this register to trigger an interrupt when LY reg and this reg are the same value
    stat: StatReg,      //$FF41 - LCD status register
    scx: u8,            //$FF43 - Scrolling x register
    scy: u8,            //$FF42 - Scrolling y register
    wx: u8,             //$FF4B - Window x position
    wy: u8,             //$FF4A - Window y position
    bgp: PaletteReg,    //$FF47 - Background palette data - Non-CGB Mode only
    obp0: PaletteReg,   //$FF48 - Object palette 0 data - Non-CGB Mode only
    obp1: PaletteReg,   //$FF49 - Object palette 1 data - Non-CGB Mode only
}

impl PpuRegisters {
    fn new() -> Self {
        Self {
            lcdc: LcdcReg::new(),
            ly: 0,
            lyc: 0,
            stat: StatReg::new(),
            scx: 0,
            scy: 0,
            wx: 0,
            wy: 0,
            bgp: PaletteReg::new(),
            obp0: PaletteReg::new(),
            obp1: PaletteReg::new(),
        }
    }
}

enum PaletteColors {
    White,
    LightGrey,
    DarkGrey,
    Black,
}

/**
 * Represents a register that contains color id for palettes. This can be used 
 * for object and background palette registers
 */
struct PaletteReg {
    color_id_0: PaletteColors,
    color_id_1: PaletteColors,
    color_id_2: PaletteColors,
    color_id_3: PaletteColors,
}

impl PaletteReg {
    fn new() -> Self {
        Self {
            color_id_0: PaletteColors::White,
            color_id_1: PaletteColors::LightGrey,
            color_id_2: PaletteColors::DarkGrey,
            color_id_3: PaletteColors::Black,
        }
    }
}

/**
 * Represents the pixel fetcher in the gameboy. It'll house all the things 
 * necessary to fetch sprite, bg, and window pixels
 */
struct PixelFetcher {

}

