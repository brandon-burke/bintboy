use super::mbc;

#[derive(Debug)]
pub enum MBC {
    RomOnly,
    MBC1(mbc::MBC1),
    MBC2,
    MBC3,
    MBC5,
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