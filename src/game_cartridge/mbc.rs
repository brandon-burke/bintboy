use super::enums::ROMSize;

#[derive(Debug)]
pub struct MBC1 {
    pub ram_enable: bool,
    pub rom_bank_num: u8,
    pub ram_bank_num: u8,       //This is also can be used as the upper 2bits of the rom bank number
    pub banking_mode_sel: u8,
}

impl MBC1 {
    pub fn new() -> Self {
        Self {
            ram_enable: false,
            rom_bank_num: 0,
            ram_bank_num: 0,
            banking_mode_sel: 0,
        }
    }

    /**
     * This will write to the ram enable register. Only values of 0xA written to
     * the lower 4 bits will enable the ram. Any other value will disable it
     */
    pub fn write_ram_enable(&mut self, value: u8) {
        self.ram_enable = (value & 0xF) == 0xA;
    }

    /**
     * This will write and change the current rom bank number for the switchable
     * rom bank in the Game Boy. The rom bank num cannot be written the value of 
     * 0, as rom bank 0 is permanately mapped to the NON-switchable bank 0 in the Game Boy. BUT a 
     * weird quirk exists, where you can make this happen and the game cartridge's 
     * bank 0 is copied to the switchable rom bank in the Game Boy
     */
    pub fn write_rom_bank_num(&mut self, value: u8, bank_bit_mask: u16, rom_size: &ROMSize) {
        let mut rom_bank_num = value & bank_bit_mask as u8;

        //Weird quirk always accounting for the total 5 bits
        if (value & 0x1F) == 0 {
            rom_bank_num = 1;
        }

        //Accouting for roms that are 1MiB+
        if *rom_size >= ROMSize::_1MiB {
            rom_bank_num += self.ram_bank_num << 5;
        }

        //Finalized rom bank num
        self.rom_bank_num = rom_bank_num;
    }

    /**
     * This will write to the second 2bit banking register. Depending on the 
     * banking select mode, this will determine what these 2 bits do.
     */
    pub fn write_ram_bank_num(&mut self, value: u8) {
        let ram_bank_num = value & 0x3;
        self.ram_bank_num = ram_bank_num;


        // (MBC::MBC1, RAMSize::_32KiB) => {
        //     if self.mbc_reg.banking_mode_sel_reg == 1 {
        //         let ram_bank_num = data_to_write & 0x3;
        //         self.game_data.ram_banks[self.mbc_reg.ram_bank_num_reg as usize] = self.sram;
        //         self.mbc_reg.ram_bank_num_reg = ram_bank_num;
        //         self.sram = self.game_data.ram_banks[ram_bank_num as usize];
        //     } else {
        //         self.game_data.ram_banks[self.mbc_reg.ram_bank_num_reg as usize] = self.sram;
        //         self.sram = self.game_data.ram_banks[0];
        //         self.mbc_reg.ram_bank_num_reg = 0;
        //     }
        // }
    }

    pub fn write_banking_mode_sel(&mut self, value: u8) {
        self.banking_mode_sel = value & 0x1;
    }
    
    pub fn is_ram_enabled(&self) -> bool {
        return self.ram_enable;
    }
}

#[derive(Debug)]
pub struct MBC2 {

}

impl MBC2 {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn write_0x0000_to_0x1fff(&mut self, value: u8) {
        todo!()
    }

    pub fn write_0x2000_to_0x3fff(&mut self, value: u8) {
        todo!()
    }

    pub fn write_0x4000_to_0x5fff(&mut self, value: u8) {
        todo!()
    }

    pub fn write_0x6000_to_0x7fff(&mut self, value: u8) {
        todo!()
    }

    pub fn is_ram_enabled(&self) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct MBC3 {

}

impl MBC3 {
    pub fn new() -> Self {
        Self {
            
        }
    }

    pub fn write_0x0000_to_0x1fff(&mut self, value: u8) {
        todo!()
    }

    pub fn write_0x2000_to_0x3fff(&mut self, value: u8) {
        todo!()
    }

    pub fn write_0x4000_to_0x5fff(&mut self, value: u8) {
        todo!()
    }

    pub fn write_0x6000_to_0x7fff(&mut self, value: u8) {
        todo!()
    }

    pub fn is_ram_enabled(&self) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct MBC5 {

}

impl MBC5 {
    pub fn new() -> Self {
        Self {
            
        }
    }

    pub fn write_0x0000_to_0x1fff(&mut self, value: u8) {
        todo!()
    }

    pub fn write_0x2000_to_0x3fff(&mut self, value: u8) {
        todo!()
    }

    pub fn write_0x4000_to_0x5fff(&mut self, value: u8) {
        todo!()
    }

    pub fn write_0x6000_to_0x7fff(&mut self, value: u8) {
        todo!()
    }

    pub fn is_ram_enabled(&self) -> bool {
        todo!()
    }
}