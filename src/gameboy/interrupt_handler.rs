use core::panic;

use crate::gameboy::binary_utils;

const MACHINE_CYCLE: u8 = 4;

#[derive(Debug, PartialEq)]
pub enum Interrupt {
    VBlank,
    LcdStatus,
    TimerOverflow,
    SerialLink,
    Joypad,
    Idle,
}

pub struct InterruptHandler {
    pub ime_flag: bool,
    pub ie_reg: u8,
    pub if_reg: u8,
    pub handling_interrupt: Interrupt,
    pub handling_isr: bool,
    cycles_since_ime_flag_set: u8,
    machine_cycle: u8,
}

impl InterruptHandler {
    pub fn new() -> Self {
        Self { 
            ime_flag: false, 
            ie_reg: 0, 
            if_reg: 0xE1, 
            handling_interrupt: Interrupt::Idle,
            handling_isr: false,
            cycles_since_ime_flag_set: 0, // How many cycles since the ime flag was set
            machine_cycle: 0,
        }
    }

    /**
     * This is going to be called every clk cycle. This is so ugly but
     * since I can't pass the memory object to the interrupt handler because of borrowing issues
     * I have to do this. I'm going to have to refactor this later
     */
    pub fn cycle(&mut self, pc: &mut u16) -> u8 {
        if self.cycles_since_ime_flag_set > 0 {
            self.cycles_since_ime_flag_set -= 1;

            if self.cycles_since_ime_flag_set == 0 {
                self.ime_flag = true;
            }
        }

        let enabled_and_requested_interrupts = (self.ie_reg & self.if_reg) & 0x1F;
        if (self.ime_flag && enabled_and_requested_interrupts != 0) || self.handling_isr {
            self.machine_cycle += 1;
            self.isr_routine(pc);
        }

        return self.machine_cycle;
    }

    fn isr_routine(&mut self, pc: &mut u16) {
        match self.machine_cycle {
            1 => {
                self.handling_isr = true;
                for bit_pos in 0..=4 {
                    if binary_utils::get_bit(self.if_reg, bit_pos) != 0 {
                        self.if_reg = binary_utils::reset_bit(self.if_reg, bit_pos); 
                        self.handling_interrupt = match bit_pos {
                            0 => Interrupt::VBlank,
                            1 => Interrupt::LcdStatus,
                            2 => Interrupt::TimerOverflow,
                            3 => Interrupt::SerialLink,
                            4 => Interrupt::Joypad,
                            _ => panic!("Invalid interrupt vector"),
                        };
                        break;
                    }
                }   
                self.ime_flag = false;
            },
            2 => (), 
            3 => {
                // *sp -= 1;
                // memory.write_byte(*sp, (*pc >> 8) as u8);
            },
            4 => {
                // *sp -= 1;
                // memory.write_byte(*sp, *pc as u8);
            },
            5 => {
                *pc = match self.handling_interrupt {
                    Interrupt::VBlank => 0x0040,        //VBLANK
                    Interrupt::LcdStatus => 0x0048,     //LCD STATUS
                    Interrupt::TimerOverflow => 0x0050, //TIMEROVERFLOW
                    Interrupt::SerialLink => 0x0058,    //SERIAL LINK
                    Interrupt::Joypad => 0x0060,        //JOYPAD
                    _ => panic!("Invalid interrupt vector"),
                };
                self.machine_cycle = 0;
                self.handling_isr = false;
            },
            _ => panic!("Invalid machine cycle for interrupt handling"),
        }
    }

    /**
     * This should only be called by the EI instruction. It will set the ime flag to true
     * but will not enable interrupts until the next instruction is executed
     */
    pub fn enable_ime_flag(&mut self) {
        self.cycles_since_ime_flag_set = MACHINE_CYCLE;
    }

    /**
     * This should only be called by the DI instruction. It will set the ime flag to false.
     * This will disable interrupts immediately
     */
    pub fn disable_ime_flag(&mut self) {
        self.ime_flag = false;
    }

    pub fn write_ie_reg(&mut self, data_to_write: u8) {
        self.ie_reg = (self.ie_reg & 0xE0) | (data_to_write & 0x1F); 
    }

    pub fn read_ie_reg(&self) -> u8 {
        self.ie_reg
    }

    pub fn write_if_reg(&mut self, data_to_write: u8) {
        self.if_reg = (self.if_reg & 0xE0) | (data_to_write & 0x1F);
    }

    /**
     * Returning the Interrupt Flags register values. Only the 5 lower bits 
     * are R/W. The others always return 1;A
     */
    pub fn read_if_reg(&self) -> u8 {
        self.if_reg | 0xE0
    }
}
