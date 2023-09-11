use core::panic;

use crate::{cpu_state::Status, memory::Memory};
use crate::binary_utils;

struct InterruptHandler {
    ime_flag: bool,
    ie_reg: u8,
    if_reg: u8,
    handling_interrupt: bool,
    interrupt_being_handled: u8,
}

impl InterruptHandler {
    fn handle_interrupt(&mut self, memory: &mut Memory, pc: &mut u16, sp: &mut u16, machine_cycle: u8) -> Status {
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
}
