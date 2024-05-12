pub mod mbc;
mod enums;

use std::{fs::File, io::Read};

use self::enums::{ROMSize, MBC};

pub struct GameCartridge {
    pub rom_banks: Vec<[u8; 0x4000]>,
    pub ram_banks: Vec<[u8; 0x2000]>,
    pub mbc: MBC,
}

impl GameCartridge {
    pub fn new() -> Self {
        Self {
            rom_banks: vec![],
            ram_banks: vec![],
            mbc: MBC::RomOnly,
        }
    }

    /**
     * 
     */
    pub fn is_ram_enabled(&self) -> bool {
        match self.mbc {
            MBC::RomOnly => false,
            MBC::MBC1(_) => todo!(),
            MBC::MBC2(_) => todo!(),
            MBC::MBC3(_) => todo!(),
            MBC::MBC5(_) => todo!(),
        }
    }

    pub fn rom_size(&self) -> ROMSize {
        match self.rom_banks[0][0x148] {
            0x0 => ROMSize::_32KiB,
            0x1 => ROMSize::_64KiB,
            0x2 => ROMSize::_128KiB,
            0x3 => ROMSize::_256KiB,
            0x4 => ROMSize::_512KiB,
            0x5 => ROMSize::_1MiB,
            0x6 => ROMSize::_2MiB,
            0x7 => ROMSize::_4MiB,
            0x8 => ROMSize::_8MiB,
            _ => panic!("Error unsupported number of banks")
        }
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

    fn num_of_ram_banks(&self) -> u8 {
        match self.rom_banks[0][0x149] {
            0x0 => 0,
            0x1 => panic!("0x1 is a unused ram size"),
            0x2 => 1,
            0x3 => 4,
            0x4 => 16,
            0x5 => 8,
        }
    }

    pub fn num_of_rom_banks(&self) -> u16 {
        match self.rom_banks[0][0x148] {
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
        match self.num_of_rom_banks() {
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
    pub fn load_cartridge(&mut self, file_path: &str) {
        let mut rom_file = File::open(file_path).expect("File not found");

        //Setting up how many 16KB banks the rom has
        for _ in 0..self.num_of_rom_banks() {
            self.rom_banks.push([0; 0x4000]);
        }

        //Loading all the game data into the rom banks
        let mut byte_count = 0;
        let mut bank_num = 0;
        for byte in rom_file.bytes() {
            self.rom_banks[bank_num][byte_count] = match byte {
                Ok(byte_value) => byte_value,
                Err(e) => panic!("Error reading rom on bank: {bank_num} and byte: {byte_count}\n\n {e}"),
            };

            byte_count += 1;
            if byte_count == 0x4000 {
                byte_count = 0;
                bank_num += 1;
            }
        }

        //Creating the 8KB ram banks
        for _ in 0..self.num_of_ram_banks() {
            self.ram_banks.push([0; 0x2000]);
        }

        //Setting the MBC controller type
        self.mbc = match self.rom_banks[0][0x147] {
            0x00 => MBC::new(0),
            0x01 ..= 0x03 => MBC::new(1),
            0x05 ..= 0x06 => MBC::new(2),
            0x0F ..= 0x13 => MBC::new(3),
            0x19 ..= 0x1E => MBC::new(5),
            _ => panic!("Come on man I don't got time to support this MBC type"),
        };
    }
}