use super::mbc;


pub const NINTENDO_LOGO: [u8; 48] = [0xCE,0xED,0x66,0x66,0xCC,0x0D,0x00,0x0B,0x03,0x73,0x00,0x83,0x00,0x0C,0x00,0x0D,
                                     0x00,0x08,0x11,0x1F,0x88,0x89,0x00,0x0E,0xDC,0xCC,0x6E,0xE6,0xDD,0xDD,0xD9,0x99,
                                     0xBB,0xBB,0x67,0x63,0x6E,0x0E,0xEC,0xCC,0xDD,0xDC,0x99,0x9F,0xBB,0xB9,0x33,0x3E];


#[derive(Debug)]
pub enum MBC {
    RomOnly,
    MBC1(mbc::MBC1),
    MBC2(mbc::MBC2),
    MBC3(mbc::MBC3),
    MBC5(mbc::MBC5),
}

impl MBC {

    /**
     * Helps creating a new MBC according to the number you provided it.
     */
    pub fn new(mbc_num: u8) -> Self {
        match mbc_num {
            0 => MBC::RomOnly,
            1 => MBC::MBC1(mbc::MBC1::new()),
            2 => MBC::MBC2(mbc::MBC2::new()),
            3 => MBC::MBC3(mbc::MBC3::new()),
            5 => MBC::MBC5(mbc::MBC5::new()),
            x => panic!("Error creating a new MBC because we don't support MBC type {x}"),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum RAMSize {
    _0KiB,
    _8KiB,
    _32KiB,
    _64KiB,
    _128KiB,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum ROMSize {
    _32KiB,
    _64KiB,
    _128KiB,
    _256KiB,
    _512KiB,
    _1MiB,
    _2MiB,
    _4MiB,
    _8MiB,
}