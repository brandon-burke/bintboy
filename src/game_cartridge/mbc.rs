use super::{RAMSize, ROMSize};

/**
 * This trait gives a general approach to creating a MBC controller. These address
 * ranges will cover MBC1-MBC5. But note, not every MBC will use the exact address
 * range, they might only use part of it. So you might need to have some of these
 * functions route to the same functionalities.
 */
pub trait MBCController {
    fn write_0x0000_to_0x1fff(&mut self, value: u8);
    fn write_0x2000_to_0x3fff(&mut self, value: u8);
    fn write_0x4000_to_0x5fff(&mut self, value: u8);
    fn write_0x6000_to_0x7fff(&mut self, value: u8);
}

pub struct MBC1 {
    ram_enable: bool,
    rom_bank_num: u8,
    ram_bank_num: u8,       //This is also can be used as the upper 2bits of the rom bank number
    banking_mode_sel: u8,
    ram_size: RAMSize,
    rom_size: ROMSize,
    bank_bit_mask: u16,
}

impl MBCController for MBC1 {
    /**
     * This will write to the ram enable register. Only values of 0xA written to
     * the lower 4 bits will enable the ram. Any other value will disable it
     */
    fn write_0x0000_to_0x1fff(&mut self, value: u8) {
        self.ram_enable = (value & 0xF) == 0xA;
    }

    /**
     * This will write and change the current rom bank number for the switchable
     * rom bank in the Game Boy. The rom bank num cannot be written the value of 
     * 0, as rom bank 0 is permenately mapped to the NON-switchable bank 0 in the Game Boy. BUT a 
     * weird quirk exists, where you can make this happen and the game cartridge's 
     * bank 0 is copied to the switchable rom bank in the Game Boy
     */
    fn write_0x2000_to_0x3fff(&mut self, value: u8) {
        let mut rom_bank_num = value & self.bank_bit_mask as u8;

        //Weird quirk always accounting for the total 5 bits
        if (value & 0x1F) == 0 {
            rom_bank_num = 1;
        }

        //Accouting for roms that are 1MiB+
        if self.rom_size >= ROMSize::_1MiB {
            rom_bank_num += self.ram_bank_num << 5;
        }

        // self.rom_bank_num_reg = rom_bank_num;
        // if self.banking_mode_sel == 1 {
        //     match rom_bank_num {
        //         0x20 | 0x40 | 0x60 => self.rom_bank_0 = self.game_data.rom_banks[rom_bank_num as usize],
        //         _ => self.rom_bank_x = self.game_data.rom_banks[rom_bank_num as usize],
        //     }
        // } else {
        //     self.rom_bank_x = self.game_data.rom_banks[rom_bank_num as usize];
        // }
    }

    fn write_0x4000_to_0x5fff(&mut self, value: u8) {
        todo!()
    }

    fn write_0x6000_to_0x7fff(&mut self, value: u8) {
        todo!()
    }
}


// pub struct MBCReg {
//     pub mbc_type: MBC,
//     pub ram_enable_reg: bool,
//     pub ram_bank_num_reg: u8,
//     pub rom_bank_num_reg: u8,
//     pub banking_mode_sel_reg: u8,
//     pub bank_bit_mask: u16,
//     pub ram_size: RAMSize,
//     pub rom_size: ROMSize,
// }