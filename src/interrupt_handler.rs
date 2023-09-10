use core::panic;

use crate::{cpu_state::Status, memory::Memory};

struct InterruptHandler {
    ime_flag: bool,
    interrupt_enable_reg: u8,
    interrupt_flag_reg: u8,
    handling_interrupt: bool,
}

impl InterruptHandler {
    fn handle_interrupt(&mut self, memory: &mut Memory, pc: u16, sp: &mut u16, machine_cycle: u8) -> Status {
        let enabled_and_requested_interrupts = self.interrupt_enable_reg & self.interrupt_flag_reg;
        if (self.ime_flag && enabled_and_requested_interrupts != 0) || self.handling_interrupt {
            match machine_cycle {
                1 | 2 => {
                    self.handling_interrupt = true;
                    self.ime_flag = false;
                    
                }, //Do nothing for the first two machine cycles but we'll just setup some flags
                3 => {
                    *sp -= 1;
                    memory.write_byte(*sp, (pc >> 8) as u8);
                },
                4 => {
                    *sp -= 1;
                    memory.write_byte(*sp, pc as u8);
                },
                5 => {
                    self.ime_flag = false;
                    self.interrupt_flag_reg = 0;
                    self.handling_interrupt = false;
                    let interrupt_vector = match enabled_and_requested_interrupts {
                        0x01 => 0x40,
                        0x02 => 0x48,
                        0x04 => 0x50,
                        0x08 => 0x58,
                        0x10 => 0x60,
                        _ => panic!("Invalid interrupt vector"),
                    };
                    return Status::Interrupt(interrupt_vector);
                },
                _ => panic!("Invalid machine cycle for interrupt handling"),
            }
        } else {
            return Status::Completed;
        }
        return Status::Running;
    }
}
