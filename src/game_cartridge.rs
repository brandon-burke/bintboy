pub mod mbc;
mod enums;

use std::{fs::File, io::{Read, Seek, SeekFrom}};

use self::enums::{RAMSize, ROMSize, MBC, NINTENDO_LOGO};

#[derive(Debug)]
pub struct GameCartridge {
    pub rom_banks: Vec<[u8; 0x4000]>,
    pub ram_banks: Vec<[u8; 0x2000]>,
    pub mbc: MBC,
    pub rom_size: ROMSize,
    pub ram_size: RAMSize,
    pub bank_bit_mask: u16,
    pub ram_bank_bit_mask: u8,
}

impl GameCartridge {
    pub fn new() -> Self {
        Self {
            rom_banks: vec![],
            ram_banks: vec![],
            mbc: MBC::RomOnly,
            rom_size: ROMSize::_32KiB,
            ram_size: RAMSize::_0KiB,
            bank_bit_mask: 0,
            ram_bank_bit_mask: 0,
        }
    }

    pub fn read_rom_bank_0(&self, idx: u16) -> u8 {
        return match &self.mbc {
            MBC::RomOnly => self.rom_banks[0][idx as usize],
            MBC::MBC1(mbc1) => {
                if mbc1.banking_mode_sel == 1 {
                    //We only care for the upper 2 bits coming from the ram bank
                    //If we actually have a rom cart large enough
                    let mut rom_bank_num = if self.rom_size >= ROMSize::_1MiB {
                        if mbc1.is_mbc1m_cart {
                            mbc1.ram_bank_num << 4
                        } else {
                            mbc1.ram_bank_num << 5
                        }
                    } else {
                        0
                    };

                    rom_bank_num &= self.bank_bit_mask as u8;

                    match rom_bank_num {
                        bank_num @ (0x20 | 0x40 | 0x60) if !mbc1.is_mbc1m_cart => {
                            self.rom_banks[bank_num as usize][idx as usize]
                        },
                        bank_num @ (0x10 | 0x20 | 0x30) if mbc1.is_mbc1m_cart => {
                            self.rom_banks[bank_num as usize][idx as usize]
                        },
                        _ => self.rom_banks[0][idx as usize],
                    }
                } else {
                    //Normal case if rom size is <=512KiB and ram size <=8KiB
                    self.rom_banks[0][idx as usize]
                }
            },
            MBC::MBC2(_) => todo!(),
            MBC::MBC3(_) => self.rom_banks[0][idx as usize],
            MBC::MBC5(_) => self.rom_banks[0][idx as usize],
        };
    }

    pub fn read_rom_bank_x(&self, idx: u16) -> u8 {
        return match &self.mbc {
            MBC::RomOnly => self.rom_banks[1][idx as usize],
            MBC::MBC1(mbc1) => {
                let mut rom_bank_num = mbc1.rom_bank_num;

                //Accouting for roms that are 1MiB+
                if self.rom_size >= ROMSize::_1MiB {
                    if mbc1.is_mbc1m_cart {
                        rom_bank_num += mbc1.ram_bank_num << 4;
                    } else {
                        rom_bank_num += mbc1.ram_bank_num << 5;
                    }
                }

                rom_bank_num &= self.bank_bit_mask as u8;

                self.rom_banks[rom_bank_num as usize][idx as usize]
            },
            MBC::MBC2(mbc2) => todo!(),
            MBC::MBC3(mbc3) => self.rom_banks[mbc3.rom_bank_num as usize][idx as usize],
            MBC::MBC5(mbc5) => self.rom_banks[mbc5.rom_bank_num as usize][idx as usize],
        };
    }

    pub fn read_sram(&self, idx: u16) -> u8 {
        let mut value = 0xFF; //Default value if we can't read SRAM

        if self.is_ram_enabled() && self.ram_size > RAMSize::_0KiB {
            value = match &self.mbc {
                MBC::RomOnly => 0xFF,
                MBC::MBC1(mbc1) => {
                    if mbc1.banking_mode_sel == 1 {
                        let ram_bank_num = mbc1.ram_bank_num & self.ram_bank_bit_mask;
                        self.ram_banks[ram_bank_num as usize][idx as usize]
                    } else {
                        self.ram_banks[0][idx as usize]
                    }
                },
                MBC::MBC2(mbc2) => todo!(),
                MBC::MBC3(mbc3) => {
                    match mbc3.ram_bank_num {
                        0x8 => mbc3.rtc_seconds,
                        0x9 => mbc3.rtc_minutes,
                        0xA => mbc3.rtc_hours,
                        0xB => mbc3.rtc_day_lower,
                        0xC => mbc3.rtc_day_upper,
                        ram_bank_num => self.ram_banks[ram_bank_num as usize][idx as usize],
                    }
                },
                MBC::MBC5(mbc5) => self.ram_banks[mbc5.sram_bank_num as usize][idx as usize],
            };
        }

        return value;
    }

    /**
     * Writing to SRAM only if it is active. If it isn't then we don't do 
     * anything
     */
    pub fn write_sram(&mut self, value: u8, idx: u16) {
        if self.is_ram_enabled() && self.ram_size > RAMSize::_0KiB {
            match &mut self.mbc {
                MBC::RomOnly => {
                    self.ram_banks[0][idx as usize] = value;
                },
                MBC::MBC1(mbc1) => {
                    if mbc1.banking_mode_sel ==  1 {
                        let ram_bank_num = mbc1.ram_bank_num & self.ram_bank_bit_mask;
                        self.ram_banks[ram_bank_num as usize][idx as usize] = value;
                    } else {
                        self.ram_banks[0][idx as usize] = value;
                    } 
                },
                MBC::MBC2(_) => todo!(),
                MBC::MBC3(mbc3) => {
                    match mbc3.ram_bank_num {
                        0x8 => mbc3.rtc_seconds = value,
                        0x9 => mbc3.rtc_minutes = value,
                        0xA => mbc3.rtc_hours = value,
                        0xB => mbc3.rtc_day_lower = value,
                        0xC => mbc3.rtc_day_upper = value,
                        ram_bank_num => self.ram_banks[ram_bank_num as usize][idx as usize] = value,
                    }
                },
                MBC::MBC5(mbc5) => self.ram_banks[mbc5.sram_bank_num as usize][idx as usize] = value,
            }
        }
    }

    /**
     * Will call the current MBC types ram enable register to see if were 
     * allowed to write or read from SRAM
     */
    fn is_ram_enabled(&self) -> bool {
        match &self.mbc {
            MBC::RomOnly => false,
            MBC::MBC1(mbc1) => mbc1.is_ram_enabled(),
            MBC::MBC2(mbc2) => mbc2.is_ram_enabled(),
            MBC::MBC3(mbc3) => mbc3.is_ram_and_timer_enabled(),
            MBC::MBC5(mbc5) => mbc5.is_ram_enabled(),
        }
    }

    /**
     * Will handle what each mbc type does writing to the address range 
     * 0x0000 - 0x1fff
     */
    pub fn write_0x0000_to_0x1fff(&mut self, value: u8) {
        match &mut self.mbc {
            MBC::RomOnly => (),
            MBC::MBC1(mbc1) => mbc1.write_ram_enable(value),
            MBC::MBC2(mbc2) => mbc2.write_0x0000_to_0x1fff(value),
            MBC::MBC3(mbc3) => mbc3.write_ram_and_timer_enable(value),
            MBC::MBC5(mbc5) => mbc5.write_ram_enable(value),
        }
    }

    /**
     * Will handle what each mbc type does writing to the address range 
     * 0x2000 - 0x3fff
     */
    pub fn write_0x2000_to_0x3fff(&mut self, value: u8, address: u16) {
        match &mut self.mbc {
            MBC::RomOnly => (),
            MBC::MBC1(mbc1) => mbc1.write_rom_bank_num(value, self.bank_bit_mask),
            MBC::MBC2(mbc2) => mbc2.write_0x2000_to_0x3fff(value),
            MBC::MBC3(mbc3) => mbc3.write_rom_bank_num(value, self.bank_bit_mask),
            MBC::MBC5(mbc5) => {
                if address > 0x2FFF {
                    mbc5.write_rom_bank_upper_bit(value, self.bank_bit_mask);
                } else {
                    mbc5.write_rom_bank_lower_8(value, self.bank_bit_mask);
                }
            },
        }
    }

    /**
     * Will handle what each mbc type does writing to the address range 
     * 0x4000 - 0x5fff
     */
    pub fn write_0x4000_to_0x5fff(&mut self, value: u8) {
        match &mut self.mbc {
            MBC::RomOnly => (),
            MBC::MBC1(mbc1) => mbc1.write_ram_bank_num(value),
            MBC::MBC2(mbc2) => mbc2.write_0x4000_to_0x5fff(value),
            MBC::MBC3(mbc3) => mbc3.write_ram_bank_num_or_rtc_sel(value),
            MBC::MBC5(mbc5) => mbc5.write_ram_bank_num(value),
        }
    }

    /**
     * Will handle what each mbc type does writing to the address range 
     * write_0x6000 - 0x7fff
     */
    pub fn write_0x6000_to_0x7fff(&mut self, value: u8) {
        match &mut self.mbc {
            MBC::RomOnly => (),
            MBC::MBC1(mbc1) => mbc1.write_banking_mode_sel(value),
            MBC::MBC2(mbc2) => mbc2.write_0x6000_to_0x7fff(value),
            MBC::MBC3(mbc3) => mbc3.write_latch_clock_data(value),
            MBC::MBC5(_) => (),
        }
    }

    fn rom_size(&self) -> ROMSize {
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
            0x0 => RAMSize::_0KiB,
            0x1 => panic!("unused ram size"),
            0x2 => RAMSize::_8KiB,
            0x3 => RAMSize::_32KiB,
            0x4 => RAMSize::_128KiB,
            0x5 => RAMSize::_64KiB,
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
            num => panic!("Invalid number of ram banks read from rom: {num}"),
        }
    }

    fn num_of_rom_banks(&self) -> u16 {
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

    fn bank_bit_mask(&self) -> u16 {
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

    fn ram_bank_bit_mask(&self) -> u8 {
        match self.num_of_ram_banks() {
            0 => 0x0,
            1 => 0x0,
            4 => 0x3,
            8 => 0x7,
            16 => 0xF,
            _ => panic!("Not a real num of ram banks come on man"),
        }
    }

    /**
     * Takes a file path to a Game Boy rom file and loads it into the rom struct.
     * This will separate the rom into 16KB banks.
     */
    pub fn load_cartridge(&mut self, file_path: &str) {
        let mut rom_file = File::open(file_path).expect("File not found");
        let file_size = rom_file.seek(SeekFrom::End(0)).expect("Error finding the file size");
        let num_of_banks = file_size / 0x4000;

        //Setting the file cursor back to the beginning
        rom_file.seek(SeekFrom::Start(0)).expect("Error resetting the file");

        //Setting up how many 16KB banks the rom has
        for _ in 0..num_of_banks {
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
        
        self.ram_size = self.ram_size();
        self.rom_size = self.rom_size();
        self.bank_bit_mask = self.bank_bit_mask();
        self.ram_bank_bit_mask = self.ram_bank_bit_mask();

        //Setting the MBC controller type
        self.mbc = match self.rom_banks[0][0x147] {
            0x00 => MBC::new(0),            //ROM-ONLY
            0x01 ..= 0x03 => {              //MBC1
                let mut mbc1 = MBC::new(1);
                if let MBC::MBC1(ref mut mbc1) = mbc1 {
                    mbc1.is_mbc1m_cart = self.is_mbc1m_cart();
                }
                mbc1
            },
            0x05 ..= 0x06 => MBC::new(2),   //MBC2
            0x0F ..= 0x13 => MBC::new(3),   //MBC3
            0x19 ..= 0x1E => MBC::new(5),   //MBC5
            _ => panic!("Come on man I don't got time to support this MBC type"),
        };
    }

    /**
     * Tests whether the cart is a MBC1M cart.
     */
    fn is_mbc1m_cart(&mut self) -> bool {
        if self.rom_size == ROMSize::_1MiB {
            let mut logo_idx = 0;
            for byte in &self.rom_banks[0x10] {
                if *byte == NINTENDO_LOGO[logo_idx] {
                    logo_idx += 1;

                    if logo_idx == NINTENDO_LOGO.len() {
                        return true;
                    }
                } else {
                    logo_idx = 0;
                }
            }
        }
        return false;
    }
}