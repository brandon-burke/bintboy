#[derive(Clone, Copy)]
pub enum SpritePriority{
    OverBg,     //Sprite should draw over background and window
    UnderBg,    //Background and window colors 1-3 are drawn over the Sprite
}


/* Represents whether a sprite is mirrored or not */
#[derive(Clone, Copy)]
pub enum Orientation {
    Normal,
    Mirrored,
}

/* Represents the options for a palette that a sprite can use */
#[derive(Clone, Copy)]
pub enum SpritePalette {
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
pub enum VramBank {
    Bank0,
    Bank1,
}



pub enum State {
    On,
    Off,
}

pub enum TileMapArea {
    _9800,
    _9C00,
}

pub enum TileDataArea {
    _8800,
    _8000,
}

pub enum SpriteSize {
    _8x8,
    _8x16,
}

#[derive(PartialEq)]
pub enum PpuMode {
    OamScan,        //Mode 2
    DrawingPixels,  //Mode 3
    Hblank,         //Mode 0
    Vblank,         //Mode 1
}


pub enum PaletteColors {
    White,
    LightGrey,
    DarkGrey,
    Black,
}