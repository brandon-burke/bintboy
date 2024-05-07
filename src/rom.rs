use std::{fs::File, io::{Read, Seek, SeekFrom}};
use crate::rom::MBC::*;
use crate::rom::RAMSize::{_0KiB, _128KiB, _32KiB, _64KiB, _8KiB};

pub struct GameCartridge {
    pub rom_banks: Vec<[u8; 0x4000]>,
    pub ram_banks: Vec<[u8; 0x2000]>
}

impl GameCartridge {
    pub fn new() -> Self {
        Self {
            rom_banks: vec![],
            ram_banks: vec![],
        }
    }

    pub fn cartridge_type(&self) -> MBC {
        match self.rom_banks[0][0x147] {
            0x00 => RomOnly,
            0x01 ..= 0x03 => MBC1,
            0x05 ..= 0x06 => MBC2,
            0x0F ..= 0x13 => MBC3,
            0x19 ..= 0x1E => MBC5,
            _ => panic!("Come on man I don't got time to support this MBC type"),
        }
    }

    pub fn rom_size(&self) -> u8 {
        return self.rom_banks[0][0x148];
    }

    pub fn ram_size(&self) -> RAMSize {
        match self.rom_banks[0][0x149] {
            0x0 => _0KiB,
            0x1 => panic!("unused ram size"),
            0x2 => _8KiB,
            0x3 => _32KiB,
            0x4 => _128KiB,
            0x5 => _64KiB,
            _ => panic!("Not of valid ram size")
        }
    }

    pub fn num_of_banks(&self) -> u16 {
        match self.rom_size() {
            0x0 => 2,
            0x1 => 4,
            0x2 => 8,
            0x3 => 16,
            0x4 => 32,
            0x5 => 64,
            0x6 => 128,
            0x7 => 256,
            0x8 => 512,
            _ => panic!("Error unsupported number of banks")
        }
    }

    pub fn bank_bit_mask(&self) -> u16 {
        match self.num_of_banks() {
            2 => 0x1,
            4 => 0x3,
            8 => 0x7,
            16 => 0xF,
            32 => 0x1F,
            64 => 0x3F,
            128 => 0x7F,
            256 => 0xFF,
            512 => 0x1FF,
            _ => panic!("Error: Unsupported number of ROM banks")
        }
    }

    /**
     * Takes a file path to a Game Boy rom file and loads it into the rom struct.
     * This will separate the rom into 16KB banks.
     */
    pub fn load_rom(&mut self, file_path: &str) {
        let mut rom_file = File::open(file_path).expect("File not found");
        let file_size = rom_file.seek(SeekFrom::End(0)).expect("Error finding the file size");
        let num_of_banks = file_size / 0x4000;

        //Setting the file cursor back to the beginning
        rom_file.seek(SeekFrom::Start(0)).expect("Error resetting the file");

        //Setting up how many 16KB banks the rom has
        for _ in 0..num_of_banks {
            self.rom_banks.push([0; 0x4000]);
        }

        //Loading all the game data into the rom
        let mut byte_count = 0;
        let mut bank_num = 0;
        for byte in rom_file.bytes() {
            self.rom_banks[bank_num][byte_count] = match byte {
                Ok(byte_value) => byte_value,
                Err(e) => panic!("Error reading rom on bank: {bank_num} and byte: {byte_count}\n {e}"),
            };

            byte_count += 1;
            if byte_count == 0x4000 {
                byte_count = 0;
                bank_num += 1;
            }
        }

        //Finding num of ram banks
        let num_ram_banks = match self.ram_size() {
            _0KiB => 0,
            _8KiB => 1,
            _32KiB => 4,
            _64KiB => 16,
            _128KiB => 8,
        };

        //Creating the ram banks
        for _ in 0..num_ram_banks {
            self.ram_banks.push([0; 0x2000]);
        }
    }
}

#[derive(Debug)]
pub enum MBC {
    RomOnly,
    MBC1,
    MBC2,
    MBC3,
    MBC5,
}

#[derive(Debug)]
pub enum RAMSize {
    _0KiB,
    _8KiB,
    _32KiB,
    _64KiB,
    _128KiB,
}

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