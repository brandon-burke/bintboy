pub struct Ppu {    
    tile_data_0: [u8; 0x800],   //$8000–$87FF
    tile_data_1: [u8; 0x800],   //$8800–$8FFF
    tile_data_2: [u8; 0x800],   //$9000–$97FF
    tile_map_0: [u8; 0x400],    //$9800-$9BFF
    tile_map_1: [u8; 0x400],    //$9C00-$9FFF
}

impl Ppu {

}


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