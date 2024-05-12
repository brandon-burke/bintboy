use super::mbc;



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
            x => panic!("Error creating a new MBC because we don't support MBC type {}", mbc_num),
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