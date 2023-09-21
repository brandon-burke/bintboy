use core::panic;

use crate::{cpu_state::Status, memory::Memory};
use crate::binary_utils;

pub struct InterruptHandler {
    pub ime_flag: bool,
    pub ie_reg: u8,
    pub if_reg: u8,
    pub interrupt_being_handled: u8,
    pub handling_interrupt: bool,
}

impl InterruptHandler {
    pub fn new() -> Self {
        InterruptHandler { 
            ime_flag: false, 
            ie_reg: 0, 
            if_reg: 0, 
            interrupt_being_handled: 0,
            handling_interrupt: false,
        }
    }

    pub fn handle_interrupt(&mut self, memory: &mut Memory, pc: &mut u16, sp: &mut u16, machine_cycle: u8) -> Status {
        let enabled_and_requested_interrupts = self.ie_reg & self.if_reg;
        if (self.ime_flag && enabled_and_requested_interrupts != 0) || self.handling_interrupt {
            match machine_cycle {
                1 => {
                    self.handling_interrupt = true;
                    self.ime_flag = false;
                    for bit_pos in 0..=4 {
                        if binary_utils::get_bit(self.if_reg, bit_pos) != 0 {
                            self.if_reg = binary_utils::reset_bit(self.if_reg, bit_pos); 
                            self.interrupt_being_handled = bit_pos;
                        }
                    }   
                },
                2 => (), //Do nothing for the first two machine cycles but we'll just setup some flags
                3 => {
                    *sp -= 1;
                    memory.write_byte(*sp, (*pc >> 8) as u8);
                },
                4 => {
                    *sp -= 1;
                    memory.write_byte(*sp, *pc as u8);
                },
                5 => {
                    *pc = match self.interrupt_being_handled {
                        1 => 0x0040,    //VBLANK
                        2 => 0x0048,    //LCD STATUS
                        3 => 0x0050,    //TIMEROVERFLOW
                        4 => 0x0058,    //SERIAL LINK
                        5 => 0x0060,    //JOYPAD
                        _ => panic!("Invalid interrupt vector"),
                    }
                },
                _ => panic!("Invalid machine cycle for interrupt handling"),
            }
        } else {
            return Status::Completed;
        }
        return Status::Running;
    }

    pub fn request_is_present(&self) -> bool {
        let enabled_and_requested_interrupts = self.ie_reg & self.if_reg;
        return (self.ime_flag && enabled_and_requested_interrupts != 0) || self.handling_interrupt;
    }

    /**
     * This function will be called when the IME flag is set to true
     */
    pub fn enable_ime_flag(&mut self) {
        self.ime_flag = true;
    }

    pub fn disable_ime_flag(&mut self) {
        self.ime_flag = false;
    }

    pub fn write_ie_reg(&mut self, data_to_write: u8) {
        self.ie_reg = data_to_write;
    }

    pub fn read_ie_reg(&self) -> u8 {
        self.ie_reg
    }

    pub fn write_if_reg(&mut self, data_to_write: u8) {
        self.if_reg = data_to_write;
    }

    pub fn read_if_reg(&self) -> u8 {
        self.if_reg
    }
}
