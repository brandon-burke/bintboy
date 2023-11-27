struct Ppu {
    tile_data_0: [Tile; 128],   //$8000–$87FF
    tile_data_1: [Tile; 128],   //$8800–$8FFF
    tile_data_2: [Tile; 128],   //$9000–$97FF
    tile_map_0: [u8; 0x400],    //$9800-$9BFF
    tile_map_1: [u8; 0x400],    //$9C00-$9FFF
    oam: [Sprite; 40],          //$FE00–$FE9F (Object Attribute Table) Sprite information table
    lcdc: LcdcRegister,
    ly: u8,
    lyc: u8,
    stat: StatRegister,
}

impl Ppu {

}

/*
    Represents a 8x8 square of pixels. Here we have an array of PixelRows
    where each PixelRow in the array represents the data of a row of pixels
    in the Tile
 */
struct Tile {
    pixel_rows: [TileRow; 8]
}

/*
    Represents the data to create a row of pixels.
 */
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

struct Sprite {
    y_pos: u8,
    x_pos: u8,
    tile_index: u8,
    priority: SpritePriority,
    y_flip: Orientation,
    x_flip: Orientation,
    dmg_palette: Palette,   //Non CGB Mode only
    bank: VramBank,         //CGB Mode only
    cgb_palette: Palette    //CGB Mode only
}

enum SpritePriority{
    OverBg,     //Sprite should draw over background and window
    UnderBg,    //Background and window colors 1-3 are drawn over the Sprite
}

/*
    Represents whether a sprite is mirrored or not
 */
enum Orientation {
    Normal,
    Mirrored,
}

/*
    Represents the options for a palette that a sprite can use
 */
enum Palette {
    Obp0,
    Obp1,
    Obp2,
    Obp3,
    Obp4,
    Obp5,
    Obp6,
    Obp7,
}

enum VramBank {
    Bank0,
    Bank1,
}

/*
    Represents the LCD Control register (LCDC)
 */
struct LcdcRegister {
    lcd_ppu_enable: State,
    win_tile_map_area: TileMapArea,
    win_enable: State,
    bg_win_tile_data_area: TileDataArea,
    bg_tile_map_area: TileMapArea,
    sprite_size: SpriteSize,
    sprite_enable: State,
    bg_win_priority: State,
}

struct StatRegister {
    unused_bit_7: u8,
    lyc_int_select: State,
    mode_2_int_select: State,
    mode_1_int_select: State,
    mode_0_int_select: State,
    lyc_ly_compare: State,
    ppu
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

