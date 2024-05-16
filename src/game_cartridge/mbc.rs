use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MBC1 {
    pub ram_enable: bool,
    pub rom_bank_num: u8,
    pub ram_bank_num: u8,       //This is also can be used as the upper 2bits of the rom bank number
    pub banking_mode_sel: u8,
    pub is_mbc1m_cart: bool,
}

impl MBC1 {
    pub fn new() -> Self {
        Self {
            ram_enable: false,
            rom_bank_num: 1,
            ram_bank_num: 0,
            banking_mode_sel: 0,
            is_mbc1m_cart: false,
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
     * 0, as rom bank 0 is permanently mapped to the NON-switchable bank 0 in the Game Boy. BUT a
     * weird quirk exists, where you can make this happen and the game cartridge's 
     * bank 0 is copied to the switchable rom bank in the Game Boy
     */
    pub fn write_rom_bank_num(&mut self, mut value: u8, mut bank_bit_mask: u16) {
        let mut max_bit_mask = 0x1F;
        if self.is_mbc1m_cart {
            max_bit_mask = 0xF;
        }

        //Weird quirk always accounting for the total 5 bits
        if (value & 0x1F) == 0 {
            value = 1;
        }

        //Capping it to a certain value
        bank_bit_mask &= max_bit_mask;

        //Finalized rom bank num
        self.rom_bank_num = value & bank_bit_mask as u8;
    }

    /**
     * This will write to the second 2bit banking register. Depending on the 
     * banking select mode, this will determine what these 2 bits do.
     */
    pub fn write_ram_bank_num(&mut self, value: u8) {
        let ram_bank_num = value & 0x3;
        self.ram_bank_num = ram_bank_num;
    }

    pub fn write_banking_mode_sel(&mut self, value: u8) {
        self.banking_mode_sel = value & 0x1;
    }
    
    pub fn is_ram_enabled(&self) -> bool {
        return self.ram_enable;
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct MBC3 {
    pub ram_and_timer_enable: bool,
    pub rom_bank_num: u8,
    pub ram_bank_num: u8,
    pub rtc_seconds: u8,
    pub rtc_minutes: u8,
    pub rtc_hours: u8,
    pub rtc_day_lower: u8,
    pub rtc_day_upper: u8,
}

impl MBC3 {
    pub fn new() -> Self {
        Self {
            ram_and_timer_enable: false,
            rom_bank_num: 1,
            ram_bank_num: 0,
            rtc_seconds: 0,
            rtc_minutes: 0,
            rtc_hours: 0,
            rtc_day_lower: 0,
            rtc_day_upper: 0,
        }
    }

    pub fn write_ram_and_timer_enable(&mut self, value: u8) {
        self.ram_and_timer_enable = (value & 0xF) == 0xA; 
    }

    pub fn write_rom_bank_num(&mut self, value: u8, bit_bank_mask: u16) {
        self.rom_bank_num = value;
        self.rom_bank_num &= bit_bank_mask as u8;
        if self.rom_bank_num == 0 {
            self.rom_bank_num = 1;
        }
    }

    pub fn write_ram_bank_num_or_rtc_sel(&mut self, value: u8) {
        self.ram_bank_num = match value {
            0x8..=0xC => value,
            _ => value & 0x3,
        };
    }

    pub fn write_latch_clock_data(&mut self, value: u8) {
        ()
    }

    pub fn is_ram_and_timer_enabled(&self) -> bool {
        return self.ram_and_timer_enable;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MBC5 {
    pub ram_enable: bool,
    pub rom_bank_num: u16,
    pub sram_bank_num: u8,
}

impl MBC5 {
    pub fn new() -> Self {
        Self {
            ram_enable: false,
            rom_bank_num: 1,
            sram_bank_num: 0,
        }
    }

    pub fn write_ram_enable(&mut self, value: u8) {
        self.ram_enable = (value & 0xF) == 0xA;
    }

    /**
     * Writing only to the lower 8 bits of the rom bank. The 9th bit will not be
     * touched here
     */
    pub fn write_rom_bank_lower_8(&mut self, value: u8, bit_bank_mask: u16) {
        self.rom_bank_num &= 0x0100;        //Clearing out all bits except the 9th bit
        self.rom_bank_num |= value as u16;
        self.rom_bank_num &= bit_bank_mask;
    }

    /**
     * Writing only to the 9th bit of the rom bank. The other bits will not be 
     * affected
     */
    pub fn write_rom_bank_upper_bit(&mut self, value: u8, bit_bank_mask: u16) {
        self.rom_bank_num &= 0x00FF;        //Clearing out all bits except the lower 8
        self.rom_bank_num |= ((value & 0x1) as u16) << 8;
        self.rom_bank_num &= bit_bank_mask;
    }

    pub fn write_ram_bank_num(&mut self, value: u8) {
        self.sram_bank_num = value & 0x0F;
    }

    pub fn is_ram_enabled(&self) -> bool {
        return self.ram_enable; 
    }
}