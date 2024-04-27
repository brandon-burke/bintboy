pub mod cpu_state;

use core::panic;
use crate::gameboy::interrupt_handler::{self, Interrupt};
use crate::gameboy::Memory;
use crate::gameboy::opcodes::{OPCODE_MACHINE_CYCLES, PREFIX_OPCODE_MACHINE_CYCLES};
use crate::gameboy::binary_utils::{self, split_16bit_num, build_16bit_num};
use self::cpu_state::{CpuState, Status};
use crate::gameboy::constants::{MACHINE_CYCLE, PREFIX_OPCODE};

#[derive(Debug)]
pub struct Cpu {
    pub a: u8,              //Accumulator Register
    pub b: u8,              //General Purpose Register
    pub c: u8,              //General Purpose Register
    pub d: u8,              //General Purpose Register
    pub e: u8,              //General Purpose Register
    pub f: u8,              //Flags Register        This is actually dependent on the header checksum with normal DMG gameboy
    pub h: u8,              //General Purpose Register
    pub l: u8,              //General Purpose Register
    pub sp: u16,            //Stack Pointer Register
    pub pc: u16,            //Program Counter Register
    pub cpu_state: CpuState,    //Let's us know the current state of the CPU
    cpu_clk_cycles: u8,     //Keeps track of how many cpu clk cycles have gone by
    pub current_opcode: u8,     //Keeps track of the current worked on opcode
}

impl Cpu {
    pub fn new() -> Cpu {
        //Note these initial values depend on the gameboy model.
        //We are assuming DMG
        Cpu { 
            a: 0x01, 
            b: 0x00, 
            c: 0x13, 
            d: 0x00, 
            e: 0xD8, 
            f: 0xB0, 
            h: 0x01, 
            l: 0x4D, 
            sp: 0xFFFE, 
            pc: 0x0100,
            cpu_state: CpuState::Fetch,
            cpu_clk_cycles: 0,
            current_opcode: 0x00,
        }
    }

    pub fn cycle(&mut self, memory: &mut Memory, is_blargg_test: bool) {
        /* Have to wait 1 machine cycle before we do anywork */
        self.cpu_clk_cycles += 1;
        if self.cpu_clk_cycles >= MACHINE_CYCLE {
            self.cpu_clk_cycles = 0;
        } else {
            return;
        }

        
        // if self.pc == 0xC7D2 {
        //     println!("made it");
        //     dbg!(&self);
        //     loop {
        //
        //     }
        // }

        // if self.pc == 0xCB35 {
        //     println!("made it");
        //     // dbg!(&self);
        //     loop {
        // 
        //     }
        // }

        //Depending on what state you are in you have to do the work that corresponds to it
        match self.cpu_state.clone() {
            CpuState::Fetch => {
                if self.pc == 0xC3F3 {
                    println!("At 0xC3F3");
                }

                if self.pc == 0xC3F8 {
                    println!("At 0xC3F8");
                }


                self.current_opcode = self.fetch(memory);

                if self.current_opcode == 0x76 {
                    println!("HALT");
                }

                if self.current_opcode == 0x10 {
                    println!("STOP");
                }

                if self.current_opcode == 0x40 && !is_blargg_test {
                    dbg!(&self);
                }

                
                
                if self.current_opcode == PREFIX_OPCODE {
                    self.cpu_state = CpuState::FetchPrefix;
                } else {
                    self.cpu_state = CpuState::Execute { machine_cycle: 0, temp_reg: 0, is_prefix: false };

                    if OPCODE_MACHINE_CYCLES[self.current_opcode as usize] == 1 {
                        match self.exexute(memory, 1, &mut 0) {
                            Status::Completed => {
                                if self.current_opcode == 0x76 {
                                    self.cpu_state = CpuState::Halt;
                                } else {
                                    self.cpu_state = CpuState::Fetch;
                                }
                            }
                            Status::Running => (),
                            Status::Error => panic!("Error Executing opcode"),
                        }
                    }
                }
            },
            CpuState::FetchPrefix => {
                self.current_opcode = self.fetch(memory);
                self.cpu_state = CpuState::Execute { machine_cycle: 0, temp_reg: 0, is_prefix: true };

                if PREFIX_OPCODE_MACHINE_CYCLES[self.current_opcode as usize] == 2 { //2 b/c you always have to fetch the prefix
                    match self.exexute_prefix(memory, 1) {
                        Status::Completed => {
                            self.cpu_state = CpuState::Fetch;
                        },
                        Status::Running => (),
                        Status::Error => panic!("Error Executing opcode"),
                    }
                }
            },
            CpuState::Execute { machine_cycle, mut temp_reg, is_prefix } => {
                let execute_status: Status;
                match is_prefix {
                    true => execute_status = self.exexute_prefix(memory, machine_cycle + 1),
                    false => execute_status = self.exexute(memory, machine_cycle + 1, &mut temp_reg),
                }

                match execute_status {
                    Status::Completed => {
                        self.cpu_state = CpuState::Fetch;
                    },
                    Status::Running => (),
                    Status::Error => panic!("Error Executing opcode"),
                }
                
                let t_reg = temp_reg;
                match &mut self.cpu_state {
                    CpuState::Execute { machine_cycle, temp_reg, .. } => {
                        *machine_cycle += 1;
                        *temp_reg = t_reg;
                    },
                    _ => (),
                }
            },
            CpuState::Halt => {
                let enabled_and_requested_interrupts = (memory.interrupt_handler.ie_reg & memory.interrupt_handler.if_reg) & 0x1F;
                if enabled_and_requested_interrupts != 0 {
                    self.cpu_state = CpuState::Fetch;
                }
            },
        }
    }

    /**
     * Retrieving the next opcode from memory
     */
    pub fn fetch(&mut self, memory: &Memory) -> u8 {
        let opcode = memory.read_byte(self.pc);
        self.pc += 1;
        return opcode;
    }

    /**
    * Given an opcode it will execute the instruction of the opcode
    */
    pub fn exexute(&mut self, memory: &mut Memory, machine_cycle: u8, temp_reg: &mut u16) -> Status {
        match self.current_opcode {
            0x00 => Cpu::nop(machine_cycle),                                                                                //NOP
            0x01 => Cpu::ld_r16_u16(memory, &mut self.b, &mut self.c, &mut self.pc, machine_cycle),     //LD_BC_U16
            0x02 => Cpu::ld_r16_a(memory, self.a, self.b, self.c, machine_cycle),                       //LD_BC_A
            0x03 => Cpu::inc_r16(&mut self.b, &mut self.c, machine_cycle),                              //INC_BC
            0x04 => Cpu::inc_r8(&mut self.f, &mut self.b, machine_cycle),                               //INC_B
            0x05 => Cpu::dec_r8(&mut self.f, &mut self.b, machine_cycle),                               //DEC_B
            0x06 => Cpu::ld_r8_u8(memory, &mut self.b, &mut self.pc, machine_cycle),                    //LD_B_U8
            0x07 => Cpu::rlca(&mut self.f, &mut self.a, machine_cycle),                                 //RLCA
            0x08 => Cpu::ld_u16_sp(memory, &mut self.pc, self.sp, machine_cycle, temp_reg),             //LD_U16_SP
            0x09 => Cpu::add_hl_r16(&mut self.f, self.b, self.c, &mut self.h, &mut self.l, machine_cycle),  //ADD_HL_BC may not be the most cycle accurate
            0x0A => Cpu::ld_a_r16(memory, &mut self.a, self.b, self.c, machine_cycle),                  //LD_A_(BC)
            0x0B => Cpu::dec_r16(&mut self.b, &mut self.c, machine_cycle),                              //DEC_BC may not be the most accurate in cycles,
            0x0C => Cpu::inc_r8(&mut self.f, &mut self.c, machine_cycle),                               //INC_C
            0x0D => Cpu::dec_r8(&mut self.f, &mut self.c, machine_cycle),                               //DEC_C
            0x0E => Cpu::ld_r8_u8(memory, &mut self.c, &mut self.pc, machine_cycle),                    //LD_C_U8
            0x0F => Cpu::rrca(&mut self.f, &mut self.a, machine_cycle),                                 //RRCA         
            0x10 => Cpu::stop(&mut self.pc, machine_cycle),                                                                        //STOP
            0x11 => Cpu::ld_r16_u16(memory, &mut self.d, &mut self.e, &mut self.pc, machine_cycle),     //LD_DE_U16
            0x12 => Cpu::ld_r16_a(memory, self.a, self.d, self.e, machine_cycle),                       //LD_(DE)_A
            0x13 => Cpu::inc_r16(&mut self.d, &mut self.e, machine_cycle),                              //INC_DE
            0x14 => Cpu::inc_r8(&mut self.f, &mut self.d, machine_cycle),                               //INC_D
            0x15 => Cpu::dec_r8(&mut self.f, &mut self.d, machine_cycle),                               //DEC_D
            0x16 => Cpu::ld_r8_u8(memory, &mut self.d, &mut self.pc, machine_cycle),                              //LD_D_U8
            0x17 => Cpu::rla(&mut self.f, &mut self.a, machine_cycle),                                  //RLA
            0x18 => Cpu::jr_i8(memory, &mut self.pc, machine_cycle, temp_reg),                          //JR_i8
            0x19 => Cpu::add_hl_r16(&mut self.f, self.d, self.e, &mut self.h, &mut self.l, machine_cycle),    //ADD_HL_DE
            0x1A => Cpu::ld_a_r16(memory, &mut self.a, self.d, self.e, machine_cycle),                  //LD_A_R16
            0x1B => Cpu::dec_r16(&mut self.d, &mut self.e, machine_cycle),                              //DEC_DE
            0x1C => Cpu::inc_r8(&mut self.f, &mut self.e, machine_cycle),                               //INC_E
            0x1D => Cpu::dec_r8(&mut self.f, &mut self.e, machine_cycle),                               //DEC_E
            0x1E => Cpu::ld_r8_u8(memory, &mut self.e, &mut self.pc, machine_cycle),                              //LD_E_U8
            0x1F => Cpu::rra(&mut self.f, &mut self.a, machine_cycle),                                  //RRA
            0x20 => Cpu::jr_cc_i8(memory, &mut self.pc, Cpu::get_zero_flag(self.f) == 0, machine_cycle, temp_reg),           //JR_NZ_I8                                        
            0x21 => Cpu::ld_r16_u16(memory, &mut self.h, &mut self.l, &mut self.pc, machine_cycle),       //LD_HL_U16
            0x22 => Cpu::ld_hli_a(memory, &mut self.a, &mut self.h, &mut self.l, machine_cycle),          //LD_HLI_A
            0x23 => Cpu::inc_r16(&mut self.h, &mut self.l, machine_cycle),                                //INC_HL
            0x24 => Cpu::inc_r8(&mut self.f, &mut self.h, machine_cycle),                                   //INC_H
            0x25 => Cpu::dec_r8(&mut self.f, &mut self.h, machine_cycle),                                   //DEC_H
            0x26 => Cpu::ld_r8_u8(memory, &mut self.h, &mut self.pc, machine_cycle),                        //LD_H_U8
            0x27 => Cpu::daa(&mut self.f, &mut self.a, machine_cycle),                                      //DAA 
            0x28 => Cpu::jr_cc_i8(memory, &mut self.pc, Cpu::get_zero_flag(self.f) != 0, machine_cycle, temp_reg),  //JR_Z_I8
            0x29 => Cpu::add_hl_r16(&mut self.f, self.h, self.l, &mut self.h, &mut self.l, machine_cycle),  //ADD_HL_HL
            0x2A => Cpu::ld_a_hli(memory, &mut self.a, &mut self.h, &mut self.l, machine_cycle),            //LD_A_HLI
            0x2B => Cpu::dec_r16(&mut self.h, &mut self.l, machine_cycle),                                  //DEC_HL
            0x2C => Cpu::inc_r8(&mut self.f, &mut self.l, machine_cycle),                                   //INC_L              
            0x2D => Cpu::dec_r8(&mut self.f, &mut self.l, machine_cycle),                                   //DEC_L
            0x2E => Cpu::ld_r8_u8(memory, &mut self.l, &mut self.pc, machine_cycle),                        //LD_L_U8
            0x2F => Cpu::cpl(&mut self.f, &mut self.a, machine_cycle),                                      //CPL
            0x30 => Cpu::jr_cc_i8(memory, &mut self.pc, Cpu::get_carry_flag(self.f) == 0, machine_cycle, temp_reg), //JR_NC_I8
            0x31 => Cpu::ld_sp_u16(memory, &mut self.pc, &mut self.sp, machine_cycle),                  //LD_SP_U16
            0x32 => Cpu::ld_hld_a(memory, &mut self.a, &mut self.h, &mut self.l, machine_cycle),        //LD_HLD_A
            0x33 => Cpu::inc_sp(&mut self.sp, machine_cycle),               //INC_SP
            0x34 => Cpu::inc_hl(&mut self.f, memory, &mut self.h, &mut self.l, machine_cycle),                         //INC_HL
            0x35 => Cpu::dec_hl(&mut self.f, memory, &mut self.h, &mut self.l, machine_cycle),             //DEC_HL
            0x36 => Cpu::ld_hl_u8(memory, self.h, self.l, &mut self.pc, machine_cycle),                    //LD_HL_U8
            0x37 => Cpu::scf(&mut self.f, machine_cycle),                                                   //SCF
            0x38 => Cpu::jr_cc_i8(memory, &mut self.pc, Cpu::get_carry_flag(self.f) != 0, machine_cycle, temp_reg), //JR_C_I8
            0x39 => Cpu::add_hl_sp(&mut self.f, &mut self.h, &mut self.l, &mut self.sp, machine_cycle),     //ADD_HL_SP
            0x3A => Cpu::ld_a_hld(memory, &mut self.a, &mut self.h, &mut self.l, machine_cycle),        //LD_A_HLD
            0x3B => Cpu::dec_sp(&mut self.sp, machine_cycle),                                           //DEC_SP
            0x3C => Cpu::inc_r8(&mut self.f, &mut self.a, machine_cycle),                               //INC_R8
            0x3D => Cpu::dec_r8(&mut self.f, &mut self.a, machine_cycle),               //DEC_R8    
            0x3E => Cpu::ld_r8_u8(memory, &mut self.a, &mut self.pc, machine_cycle),            //LD_A_U8
            0x3F => Cpu::ccf(&mut self.f, machine_cycle),                                       //CCF
            0x40 => Cpu::ld_r8_r8(self.b, &mut self.b, machine_cycle),                          //LD_B_B
            0x41 => Cpu::ld_r8_r8(self.c, &mut self.b, machine_cycle),                          //LD_B_C
            0x42 => Cpu::ld_r8_r8(self.d, &mut self.b, machine_cycle),                          //LD_B_D
            0x43 => Cpu::ld_r8_r8(self.e, &mut self.b, machine_cycle),                          //LD_B_E
            0x44 => Cpu::ld_r8_r8(self.h, &mut self.b, machine_cycle),                          //LD_B_H
            0x45 => Cpu::ld_r8_r8(self.l, &mut self.b, machine_cycle),                          //LD_B_L
            0x46 => Cpu::ld_r8_hl(memory, self.h, self.l, &mut self.b, machine_cycle),          //LD_B_(HL)
            0x47 => Cpu::ld_r8_r8(self.a, &mut self.b, machine_cycle),                          //LD_B_A
            0x48 => Cpu::ld_r8_r8(self.b, &mut self.c, machine_cycle),                          //LD_C_B
            0x49 => Cpu::ld_r8_r8(self.c, &mut self.c, machine_cycle),                          //LD_C_C
            0x4A => Cpu::ld_r8_r8(self.d, &mut self.c, machine_cycle),                          //LD_C_D
            0x4B => Cpu::ld_r8_r8(self.e, &mut self.c, machine_cycle),                          //LD_C_E
            0x4C => Cpu::ld_r8_r8(self.h, &mut self.c, machine_cycle),                          //LD_C_H
            0x4D => Cpu::ld_r8_r8(self.l, &mut self.c, machine_cycle),                          //LD_C_L
            0x4E => Cpu::ld_r8_hl(memory, self.h, self.l, &mut self.c, machine_cycle),          //LD_C_(HL)
            0x4F => Cpu::ld_r8_r8(self.a, &mut self.c, machine_cycle),                          //LD_C_A
            0x50 => Cpu::ld_r8_r8(self.b, &mut self.d, machine_cycle),                          //LD_D_B
            0x51 => Cpu::ld_r8_r8(self.c, &mut self.d, machine_cycle),                          //LD_D_C
            0x52 => Cpu::ld_r8_r8(self.d, &mut self.d, machine_cycle),                          //LD_D_D
            0x53 => Cpu::ld_r8_r8(self.e, &mut self.d, machine_cycle),                          //LD_D_E
            0x54 => Cpu::ld_r8_r8(self.h, &mut self.d, machine_cycle),                          //LD_D_H
            0x55 => Cpu::ld_r8_r8(self.l, &mut self.d, machine_cycle),                          //LD_D_L
            0x56 => Cpu::ld_r8_hl(memory, self.h, self.l, &mut self.d, machine_cycle),          //LD_D_(HL)
            0x57 => Cpu::ld_r8_r8(self.a, &mut self.d, machine_cycle),                          //LD_D_A
            0x58 => Cpu::ld_r8_r8(self.b, &mut self.e, machine_cycle),                          //LD_E_B 
            0x59 => Cpu::ld_r8_r8(self.c, &mut self.e, machine_cycle),                          //LD_E_C
            0x5A => Cpu::ld_r8_r8(self.d, &mut self.e, machine_cycle),                          //LD_E_D
            0x5B => Cpu::ld_r8_r8(self.e, &mut self.e, machine_cycle),                          //LD_E_E
            0x5C => Cpu::ld_r8_r8(self.h, &mut self.e, machine_cycle),                          //LD_E_H
            0x5D => Cpu::ld_r8_r8(self.l, &mut self.e, machine_cycle),                          //LD_E_L
            0x5E => Cpu::ld_r8_hl(memory, self.h, self.l, &mut self.e, machine_cycle),          //LD_E_(HL)
            0x5F => Cpu::ld_r8_r8(self.a, &mut self.e, machine_cycle),                          //LD_E_A
            0x60 => Cpu::ld_r8_r8(self.b, &mut self.h, machine_cycle),                          //LD_H_B
            0x61 => Cpu::ld_r8_r8(self.c, &mut self.h, machine_cycle),                          //LD_H_C
            0x62 => Cpu::ld_r8_r8(self.d, &mut self.h, machine_cycle),                          //LD_H_D
            0x63 => Cpu::ld_r8_r8(self.e, &mut self.h, machine_cycle),                          //LD_H_E
            0x64 => Cpu::ld_r8_r8(self.h, &mut self.h, machine_cycle),                          //LD_H_H
            0x65 => Cpu::ld_r8_r8(self.l, &mut self.h, machine_cycle),                          //LD_H_L
            0x66 => Cpu::ld_r8_hl(memory, self.h, self.l, &mut self.h, machine_cycle),          //LD_H_(HL)
            0x67 => Cpu::ld_r8_r8(self.a, &mut self.h, machine_cycle),                          //LD_H_A
            0x68 => Cpu::ld_r8_r8(self.b, &mut self.l, machine_cycle),                          //LD_L_B
            0x69 => Cpu::ld_r8_r8(self.c, &mut self.l, machine_cycle),                          //LD_L_C
            0x6A => Cpu::ld_r8_r8(self.d, &mut self.l, machine_cycle),                          //LD_L_D
            0x6B => Cpu::ld_r8_r8(self.e, &mut self.l, machine_cycle),                          //LD_L_E
            0x6C => Cpu::ld_r8_r8(self.h, &mut self.l, machine_cycle),                          //LD_L_H
            0x6D => Cpu::ld_r8_r8(self.l, &mut self.l, machine_cycle),                          //LD_L_L
            0x6E => Cpu::ld_r8_hl(memory, self.h, self.l, &mut self.l, machine_cycle),          //LD_L_(HL)
            0x6F => Cpu::ld_r8_r8(self.a, &mut self.l, machine_cycle),                          //LD_L_A
            0x70 => Cpu::ld_hl_r8(memory, self.b, self.h, self.l, machine_cycle),               //LD_(HL)_B
            0x71 => Cpu::ld_hl_r8(memory, self.c, self.h, self.l, machine_cycle),               //LD_(HL)_C
            0x72 => Cpu::ld_hl_r8(memory, self.d, self.h, self.l, machine_cycle),               //LD_(HL)_D
            0x73 => Cpu::ld_hl_r8(memory, self.e, self.h, self.l, machine_cycle),               //LD_(HL)_E
            0x74 => Cpu::ld_hl_r8(memory, self.h, self.h, self.l, machine_cycle),               //LD_(HL)_H
            0x75 => Cpu::ld_hl_r8(memory, self.l, self.h, self.l, machine_cycle),               //LD_(HL)_L
            0x76 => Cpu::halt(self),                                                                //HALT
            0x77 => Cpu::ld_hl_r8(memory, self.a, self.h, self.l, machine_cycle),               //LD_(HL)_A
            0x78 => Cpu::ld_r8_r8(self.b, &mut self.a, machine_cycle),                          //LD_A_B
            0x79 => Cpu::ld_r8_r8(self.c, &mut self.a, machine_cycle),                          //LD_A_C
            0x7A => Cpu::ld_r8_r8(self.d, &mut self.a, machine_cycle),                          //LD_A_D
            0x7B => Cpu::ld_r8_r8(self.e, &mut self.a, machine_cycle),                          //LD_A_E
            0x7C => Cpu::ld_r8_r8(self.h, &mut self.a, machine_cycle),                          //LD_A_H
            0x7D => Cpu::ld_r8_r8(self.l, &mut self.a, machine_cycle),                          //LD_A_L
            0x7E => Cpu::ld_r8_hl(memory, self.h, self.l, &mut self.a, machine_cycle),          //LD_A_(HL)
            0x7F => Cpu::ld_r8_r8(self.a, &mut self.a, machine_cycle),                          //LD_A_A
            0x80 => Cpu::add_a_r8(&mut self.f, self.b, &mut self.a, machine_cycle),             //ADD_A_B
            0x81 => Cpu::add_a_r8(&mut self.f, self.c, &mut self.a, machine_cycle),             //ADD_A_C
            0x82 => Cpu::add_a_r8(&mut self.f, self.d, &mut self.a, machine_cycle),             //ADD_A_D
            0x83 => Cpu::add_a_r8(&mut self.f, self.e, &mut self.a, machine_cycle),             //ADD_A_E
            0x84 => Cpu::add_a_r8(&mut self.f, self.h, &mut self.a, machine_cycle),             //ADD_A_H
            0x85 => Cpu::add_a_r8(&mut self.f, self.l, &mut self.a, machine_cycle),             //ADD_A_L
            0x86 => Cpu::add_a_hl(&mut self.f, memory, &mut self.a, self.h, self.l, machine_cycle), //ADD_A_(HL)
            0x87 => Cpu::add_a_r8(&mut self.f, self.a, &mut self.a, machine_cycle),             //ADD_A_A
            0x88 => Cpu::adc_a_r8(&mut self.f, self.b, &mut self.a, machine_cycle),             //ADC_A_B
            0x89 => Cpu::adc_a_r8(&mut self.f, self.c, &mut self.a, machine_cycle),             //ADC_A_C
            0x8A => Cpu::adc_a_r8(&mut self.f, self.d, &mut self.a, machine_cycle),             //ADC_A_D
            0x8B => Cpu::adc_a_r8(&mut self.f, self.e, &mut self.a, machine_cycle),             //ADC_A_E
            0x8C => Cpu::adc_a_r8(&mut self.f, self.h, &mut self.a, machine_cycle),             //ADC_A_H
            0x8D => Cpu::adc_a_r8(&mut self.f, self.l, &mut self.a, machine_cycle),             //ADC_A_L
            0x8E => Cpu::adc_a_hl(&mut self.f, memory, &mut self.a, self.h, self.l, machine_cycle),//ADC_A_(HL)
            0x8F => Cpu::adc_a_r8(&mut self.f, self.a, &mut self.a, machine_cycle),             //ADC_A_A
            0x90 => Cpu::sub_a_r8(&mut self.f, self.b, &mut self.a, machine_cycle),             //SUB_A_B
            0x91 => Cpu::sub_a_r8(&mut self.f, self.c, &mut self.a, machine_cycle),             //SUB_A_C,
            0x92 => Cpu::sub_a_r8(&mut self.f, self.d, &mut self.a, machine_cycle),             //SUB_A_D
            0x93 => Cpu::sub_a_r8(&mut self.f, self.e, &mut self.a, machine_cycle),             //SUB_A_E
            0x94 => Cpu::sub_a_r8(&mut self.f, self.h, &mut self.a, machine_cycle),             //SUB_A_H
            0x95 => Cpu::sub_a_r8(&mut self.f, self.l, &mut self.a, machine_cycle),             //SUB_A_L
            0x96 => Cpu::sub_a_hl(&mut self.f, memory, &mut self.a, self.h, self.l, machine_cycle),//SUB_A_HL
            0x97 => Cpu::sub_a_r8(&mut self.f, self.a, &mut self.a, machine_cycle),             //SUB_A_A
            0x98 => Cpu::sbc_a_r8(&mut self.f, self.b, &mut self.a, machine_cycle),             //SBC_A_B
            0x99 => Cpu::sbc_a_r8(&mut self.f, self.c, &mut self.a, machine_cycle),             //SBC_A_C
            0x9A => Cpu::sbc_a_r8(&mut self.f, self.d, &mut self.a, machine_cycle),             //SBC_A_D
            0x9B => Cpu::sbc_a_r8(&mut self.f, self.e, &mut self.a, machine_cycle),             //SBC_A_E
            0x9C => Cpu::sbc_a_r8(&mut self.f, self.h, &mut self.a, machine_cycle),             //SBC_A_H
            0x9D => Cpu::sbc_a_r8(&mut self.f, self.l, &mut self.a, machine_cycle),             //SBC_A_L
            0x9E => Cpu::sbc_a_hl(&mut self.f, self.h, self.l, &mut self.a, memory, machine_cycle),//SBC_A_HL
            0x9F => Cpu::sbc_a_r8(&mut self.f, self.a, &mut self.a, machine_cycle),             //SBC_A_A
            0xA0 => Cpu::and_a_r8(&mut self.f, self.b, &mut self.a, machine_cycle),             //AND_A_B    
            0xA1 => Cpu::and_a_r8(&mut self.f, self.c, &mut self.a, machine_cycle),             //AND_A_C
            0xA2 => Cpu::and_a_r8(&mut self.f, self.d, &mut self.a, machine_cycle),             //AND_A_D
            0xA3 => Cpu::and_a_r8(&mut self.f, self.e, &mut self.a, machine_cycle),             //AND_A_E
            0xA4 => Cpu::and_a_r8(&mut self.f, self.h, &mut self.a, machine_cycle),             //AND_A_H
            0xA5 => Cpu::and_a_r8(&mut self.f, self.l, &mut self.a, machine_cycle),             //AND_A_L
            0xA6 => Cpu::and_a_hl(&mut self.f, memory, &mut self.a, self.h, self.l, machine_cycle), //AND_A_HL
            0xA7 => Cpu::and_a_r8(&mut self.f, self.a, &mut self.a, machine_cycle),                 //AND_A_A
            0xA8 => Cpu::xor_a_r8(&mut self.f, self.b, &mut self.a, machine_cycle),                 //XOR_A_B
            0xA9 => Cpu::xor_a_r8(&mut self.f, self.c, &mut self.a, machine_cycle),                 //XOR_A_C
            0xAA => Cpu::xor_a_r8(&mut self.f, self.d, &mut self.a, machine_cycle),                 //XOR_A_D
            0xAB => Cpu::xor_a_r8(&mut self.f, self.e, &mut self.a, machine_cycle),                 //XOR_A_E,
            0xAC => Cpu::xor_a_r8(&mut self.f, self.h, &mut self.a, machine_cycle),                 //XOR_A_H
            0xAD => Cpu::xor_a_r8(&mut self.f, self.l, &mut self.a, machine_cycle),                 //XOR_A_L
            0xAE => Cpu::xor_a_hl(&mut self.f, memory, &mut self.a, self.h, self.l, machine_cycle), //XOR_A_HL
            0xAF => Cpu::xor_a_r8(&mut self.f, self.a, &mut self.a, machine_cycle),                 //XOR_A_A
            0xB0 => Cpu::or_a_r8(&mut self.f, self.b, &mut self.a, machine_cycle),                  //OR_A_B
            0xB1 => Cpu::or_a_r8(&mut self.f, self.c, &mut self.a, machine_cycle),                  //OR_A_C
            0xB2 => Cpu::or_a_r8(&mut self.f, self.d, &mut self.a, machine_cycle),                  //OR_A_D
            0xB3 => Cpu::or_a_r8(&mut self.f, self.e, &mut self.a, machine_cycle),                  //OR_A_E
            0xB4 => Cpu::or_a_r8(&mut self.f, self.h, &mut self.a, machine_cycle),                  //OR_A_H
            0xB5 => Cpu::or_a_r8(&mut self.f, self.l, &mut self.a, machine_cycle),                  //OR_A_L
            0xB6 => Cpu::or_a_hl(&mut self.f, memory, &mut self.a, self.h, self.l, machine_cycle),  //OR_A_HL
            0xB7 => Cpu::or_a_r8(&mut self.f, self.a, &mut self.a, machine_cycle),                  //OR_A_A
            0xB8 => Cpu::cp_a_r8(&mut self.f, self.b, self.a, machine_cycle),                       //CP_A_B
            0xB9 => Cpu::cp_a_r8(&mut self.f, self.c, self.a, machine_cycle),                       //CP_A_C
            0xBA => Cpu::cp_a_r8(&mut self.f, self.d, self.a, machine_cycle),                       //CP_A_D
            0xBB => Cpu::cp_a_r8(&mut self.f, self.e, self.a, machine_cycle),                       //CP_A_E
            0xBC => Cpu::cp_a_r8(&mut self.f, self.h, self.a, machine_cycle),                       //CP_A_H
            0xBD => Cpu::cp_a_r8(&mut self.f, self.l, self.a, machine_cycle),                       //CP_A_L
            0xBE => Cpu::cp_a_hl(&mut self.f, memory, self.a, self.h, self.l, machine_cycle),       //CP_A_HL
            0xBF => Cpu::cp_a_r8(&mut self.f, self.a, self.a, machine_cycle),                       //CP_A_A
            0xC0 => Cpu::ret_cc(memory, &mut self.sp, &mut self.pc, Cpu::get_zero_flag(self.f) == 0, machine_cycle, temp_reg),  //RET_NZ
            0xC1 => Cpu::pop(memory, &mut self.b, &mut self.c, &mut self.sp, machine_cycle),        //POP_BC
            0xC2 => Cpu::jp_cc_u16(memory, &mut self.pc, Cpu::get_zero_flag(self.f) == 0, machine_cycle, temp_reg),             //JP_NZ_U16
            0xC3 => Cpu::jp_u16(memory, &mut self.pc, machine_cycle, temp_reg),                     //JP_U16
            0xC4 => Cpu::call_cc_u16(memory, &mut self.pc, &mut self.sp, Cpu::get_zero_flag(self.f) == 0, machine_cycle, temp_reg), //CALL_NZ_U16
            0xC5 => Cpu::push_r16(memory, self.b, self.c, &mut self.sp, machine_cycle),             //PUSH_BC
            0xC6 => Cpu::add_a_u8(&mut self.f, memory, &mut self.pc, &mut self.a, machine_cycle),   //ADD_A_U8
            0xC7 => Cpu::rst_vec(memory, &mut self.sp, &mut self.pc, 0x00, machine_cycle),          //RST_00
            0xC8 => Cpu::ret_cc(memory, &mut self.sp, &mut self.pc, Cpu::get_zero_flag(self.f) != 0, machine_cycle, temp_reg),     //RET_Z
            0xC9 => Cpu::ret(memory, &mut self.sp, &mut self.pc, machine_cycle, temp_reg),          //RET
            0xCA => Cpu::jp_cc_u16(memory, &mut self.pc, Cpu::get_zero_flag(self.f) != 0, machine_cycle, temp_reg),                 //JP_Z_U16
            0xCB => Cpu::exexute_prefix(self, memory, machine_cycle),    //Need to pass the self.current opcode here as that will
            0xCC => Cpu::call_cc_u16(memory, &mut self.pc, &mut self.sp, Cpu::get_zero_flag(self.f) != 0, machine_cycle, temp_reg),
            0xCD => Cpu::call_u16(memory, &mut self.pc, &mut self.sp, machine_cycle, temp_reg),   //CALL_U16
            0xCE => Cpu::adc_a_u8(&mut self.f, memory, &mut self.a, &mut self.pc, machine_cycle),   //ADC_A_U8
            0xCF => Cpu::rst_vec(memory, &mut self.sp, &mut self.pc, 0x08, machine_cycle),
            0xD0 => Cpu::ret_cc(memory, &mut self.sp, &mut self.pc, Cpu::get_carry_flag(self.f) == 0, machine_cycle, temp_reg), //RET_NC
            0xD1 => Cpu::pop(memory, &mut self.d, &mut self.e, &mut self.sp, machine_cycle),    //POP_DE
            0xD2 => Cpu::jp_cc_u16(memory, &mut self.pc, Cpu::get_carry_flag(self.f) == 0, machine_cycle, temp_reg),        //JP_NC_U16
            0xD3 => panic!("0xD3 is an unused opcode"),
            0xD4 => Cpu::call_cc_u16(memory, &mut self.pc, &mut self.sp, Cpu::get_carry_flag(self.f) == 0, machine_cycle, temp_reg),    //CALL_NC_U16
            0xD5 => Cpu::push_r16(memory, self.d, self.e, &mut self.sp, machine_cycle),         //PUSH_DE
            0xD6 => Cpu::sub_a_u8(&mut self.f, memory, &mut self.a, &mut self.pc, machine_cycle),   //SUB_A_U8
            0xD7 => Cpu::rst_vec(memory, &mut self.sp, &mut self.pc, 0x10, machine_cycle),    //RST_10
            0xD8 => Cpu::ret_cc(memory, &mut self.sp, &mut self.pc, Cpu::get_carry_flag(self.f) != 0, machine_cycle, temp_reg),     //RET_C
            0xD9 => Cpu::reti(memory, &mut self.sp, &mut self.pc, machine_cycle, temp_reg), //RETI
            0xDA => Cpu::jp_cc_u16(memory, &mut self.pc, Cpu::get_carry_flag(self.f) != 0, machine_cycle, temp_reg),             //JP_C_U16
            0xDB => panic!("0xDB is an unused opcode"),
            0xDC => Cpu::call_cc_u16(memory, &mut self.pc, &mut self.sp, Cpu::get_carry_flag(self.f) != 0, machine_cycle, temp_reg),    //CALL_C_U16
            0xDD => panic!("0xDD is an unused opcode"),
            0xDE => Cpu::sbc_a_u8(&mut self.f, memory, &mut self.a, &mut self.pc, machine_cycle),   //SBC_A_U8
            0xDF => Cpu::rst_vec(memory, &mut self.sp, &mut self.pc, 0x18, machine_cycle),  //RST_18
            0xE0 => Cpu::ldh_u8_a(memory, &mut self.pc, self.a, machine_cycle, temp_reg),         //LDH_U8_A
            0xE1 => Cpu::pop(memory, &mut self.h, &mut self.l, &mut self.sp, machine_cycle),    //POP_HL
            0xE2 => Cpu::ldh_c_a(memory, self.a, self.c, machine_cycle),       //LDH_(0xFF00+C)_A
            0xE3 => panic!("0xE3 is an unused opcode"),
            0xE4 => panic!("0xE4 is an unused opcode"),
            0xE5 => Cpu::push_r16(memory, self.h, self.l, &mut self.sp, machine_cycle), //PUSH_HL
            0xE6 => Cpu::and_a_u8(&mut self.f, memory, &mut self.a, &mut self.pc, machine_cycle),   //AND_A_U8
            0xE7 => Cpu::rst_vec(memory, &mut self.sp, &mut self.pc, 0x20, machine_cycle),  //RST_20
            0xE8 => Cpu::add_sp_i8(&mut self.f, memory, &mut self.sp, &mut self.pc, machine_cycle),    //ADD_SP_I8
            0xE9 => Cpu::jp_hl(self.h, self.l, &mut self.pc, machine_cycle),              //JP_HL
            0xEA => Cpu::ld_u16_a(memory, &mut self.pc, self.a, machine_cycle, temp_reg),      //LD_U16_A
            0xEB => panic!("0xEB is an unused opcode"),
            0xEC => panic!("0xEC is an unused opcode"),
            0xED => panic!("0xED is an unused opcode"),
            0xEE => Cpu::xor_a_u8(&mut self.f, memory, &mut self.a, &mut self.pc, machine_cycle),   //XOR_A_U8
            0xEF => Cpu::rst_vec(memory, &mut self.sp, &mut self.pc, 0x28, machine_cycle),  //RST_28
            0xF0 => Cpu::ldh_a_u8(memory, &mut self.pc, &mut self.a, machine_cycle, temp_reg),         //LDH_A_U8
            0xF1 => {  
                let result = Cpu::pop(memory, &mut self.a, &mut self.f, &mut self.sp, machine_cycle);
                self.f &= 0xF0;
                result
            },    //POP_AF
            0xF2 => Cpu::ldh_a_c(memory, &mut self.a, self.c, machine_cycle),       //LDH_A_(0xFF00+C)
            0xF3 => Cpu::di(memory, machine_cycle),                             //DI
            0xF4 => panic!("0xF4 is an unused opcode"),
            0xF5 => Cpu::push_r16(memory, self.a, self.f & 0xF0, &mut self.sp, machine_cycle), //PUSH_AF
            0xF6 => Cpu::or_a_u8(&mut self.f, memory, &mut self.a, &mut self.pc, machine_cycle),    //OR_A_U8
            0xF7 => Cpu::rst_vec(memory, &mut self.sp, &mut self.pc, 0x30, machine_cycle),  //RST_30
            0xF8 => Cpu::ld_hl_sp_i8(&mut self.f, memory, &mut self.sp, &mut self.pc, &mut self.h, &mut self.l, machine_cycle), //LD_HL_SP_I8
            0xF9 => Cpu::ld_sp_hl(self.h, self.l, &mut self.sp, machine_cycle),            //LD_SP_HL
            0xFA => Cpu::ld_a_u16(memory, &mut self.pc, &mut self.a, machine_cycle, temp_reg),      //LD_A_U16
            0xFB => Cpu::ei(memory, machine_cycle),                             //EI
            0xFC => panic!("0xFC is an unused opcode"),
            0xFD => panic!("0xFD is an unused opcode"),
            0xFE => Cpu::cp_a_u8(&mut self.f, memory, self.a, &mut self.pc, machine_cycle),   //CP_A_U8
            0xFF => Cpu::rst_vec(memory, &mut self.sp, &mut self.pc, 0x38, machine_cycle),  //RST_38
        }
    }

    /**
     * Sets cpu flag register
     */
    fn set_flags(flag_reg: &mut u8, zero_flag: Option<bool>, negative_flag: Option<bool>, half_carry_flag: Option<bool>, carry_flag: Option<bool>) {
        match zero_flag {
            None => (),
            Some(flag) => {
                if flag {
                    *flag_reg |= 0b10000000;
                } else {
                    *flag_reg &= 0b01111111;
                }
            },
        }

        match negative_flag {
            None => (),
            Some(flag) => {
                if flag {
                    *flag_reg |= 0b01000000;
                } else {
                    *flag_reg &= 0b10111111;
                }
            }
        }

        match half_carry_flag {
            None => (),
            Some(flag) => {
                if flag {
                    *flag_reg |= 0b00100000;
                } else {
                    *flag_reg &= 0b11011111;
                }
            }
        }

        match carry_flag {
            None => (),
            Some(flag) => {
                if flag {
                    *flag_reg |= 0b00010000;
                } else {
                    *flag_reg &= 0b11101111;
                }
            }
        }
    }

    /**
     * Does absolutely nothing but consume a machine cycle and increment the pc
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn nop(machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (),
            _ => panic!("1 to many machine cycles on nop"),
        }
        return Status::Completed
    }

    /**
     * Loads the unsigned 16 bit value into the given registers
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 3
     */
    fn ld_r16_u16(memory: &Memory, upper_reg: &mut u8, lower_reg: &mut u8, pc: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => { 
                *lower_reg = memory.read_byte(*pc); 
                *pc += 1; 
            },
            2 => { 
                *upper_reg = memory.read_byte(*pc); 
                *pc += 1;
                return Status::Completed; 
            },
            _ => panic!("1 to many cycles on ld_r16_u16"),
        }
        return Status::Running;
    }

    /**
     * Store the value in register A into the byte pointed to by register r16.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 3
     */
    fn ld_r16_a(memory: &mut Memory, a_reg: u8, upper_reg: u8, lower_reg: u8, machine_cycle: u8) -> Status {
        let address: u16 = (upper_reg as u16) << 8 | lower_reg as u16;
        match machine_cycle {
            1 => memory.write_byte(address, a_reg),
            _ => panic!("1 to many cycles on ld_r16_a") 
        }

        return Status::Completed;
    }

    /**
     * Increment value in register r16 by 1. PROBABLY NOT CYCLE ACCURATE
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn inc_r16(upper_reg: &mut u8, lower_reg: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let r16 = binary_utils::build_16bit_num(*upper_reg, *lower_reg).wrapping_add(1);
                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(r16);
                *upper_reg = upper_byte;
                *lower_reg = lower_byte;
            }
            _ => panic!("1 to many cycles on inc_r16"),
        }
        return Status::Completed;
    }

    /**
     * Increment value in register r8 by 1.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn inc_r8(flag_reg: &mut u8, reg: &mut u8,  machine_cycle: u8) -> Status {
        let prev_reg_value = *reg;
        match machine_cycle {
            1 => *reg = (*reg).wrapping_add(1),
            _ => panic!("1 to many cycles on inc_r8"),
        }
        Cpu::set_flags(flag_reg, Some(*reg == 0), Some(false), Some((prev_reg_value & 0xF) + 1 > 0xF), None);
        return Status::Completed;
    }

    /**
     * Decrement value in register r8 by 1.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    pub fn dec_r8(flag_reg: &mut u8, reg: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => *reg = (*reg).wrapping_sub(1),           
            _ => panic!("1 to many cycles on dec_r8"),
        }
        Cpu::set_flags(flag_reg, Some(*reg == 0), Some(true), Some(*reg & 0xF == 0xF), None);
        return Status::Completed;
    }

    /**
     * Load value u8 into register r8
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn ld_r8_u8(memory: &Memory, reg: &mut u8, pc: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                *reg = memory.read_byte(*pc);
                *pc += 1;
            },
            _ => panic!("1 to many cycles on ld_r8_u8"),
        }
        return Status::Completed;
    }

    /**
     * Rotate register A left.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn rlca(flag_reg: &mut u8, reg_a: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                Cpu::set_flags(flag_reg, Some(false), Some(false), Some(false), Some(binary_utils::get_bit(*reg_a, 7) != 0));
                *reg_a = (*reg_a).rotate_left(1);
            }
            _ => panic!("1 to many cycles on RLCA"),
        }
        return Status::Completed;
    }

    /**
     * Store SP & $FF at address n16 and SP >> 8 at address n16 + 1.
     * 
     * MACHINE CYCLES: 5
     * INSTRUCTION LENGTH: 3
     */
    fn ld_u16_sp(memory: &mut Memory, pc: &mut u16, sp: u16, machine_cycle: u8, temp_reg: &mut u16) -> Status {
        let mut status = Status::Running;
        match machine_cycle {
            1 => { *temp_reg |= memory.read_byte(*pc) as u16; *pc += 1; },        //read lower byte ASSUMING TEMP REG TO BE 0
            2 => { *temp_reg |= (memory.read_byte(*pc) as u16) << 8; *pc += 1; }, //read upper byte 
            3 => memory.write_byte(*temp_reg, sp as u8),
            4 => {
                memory.write_byte(*temp_reg + 1, (sp >> 8) as u8);
                status = Status::Completed;
            }
            _ => panic!("1 to many cycles on LD_U16_SP"),
        }
        
        return status;
    }

    /**
     * Add the value in r16 to HL.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1 
     */
    fn add_hl_r16(flag_reg: &mut u8, upper_reg: u8, lower_reg: u8, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let reg_16 = binary_utils::build_16bit_num(upper_reg, lower_reg);
                let reg_hl = binary_utils::build_16bit_num(*reg_h, *reg_l);
                let (result, overflow) = reg_hl.overflowing_add(reg_16);
                let half_carry_overflow = (reg_hl & 0x0FFF) + (reg_16 & 0x0FFF) > 0x0FFF;

                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(result);
                *reg_h = upper_byte;
                *reg_l = lower_byte;

                Cpu::set_flags(flag_reg, None, Some(false), Some(half_carry_overflow), Some(overflow));
            }
            _ => panic!("1 to many cycles on add_hl_r16"),
        }

        return Status::Completed;
    }

    /**
     * Load value in register A from the byte pointed to by register r16.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_a_r16(memory: &Memory, reg_a: &mut u8, upper_reg: u8, lower_reg: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => *reg_a = memory.read_byte(binary_utils::build_16bit_num(upper_reg, lower_reg)),
            _ => panic!("1 to many cycles on ld_a_r16"),
        }
        return Status::Completed;
    }

    /**
    * Decrement value in register r16 by 1.
    * 
    * MACHINE CYCLES: 2
    * INSTRUCTION LENGTH: 1
    */
    fn dec_r16(upper_reg: &mut u8, lower_reg: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let r16 = binary_utils::build_16bit_num(*upper_reg, *lower_reg).wrapping_sub(1);
                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(r16);
                *upper_reg = upper_byte;
                *lower_reg = lower_byte;
            }
            _ => panic!("1 to many cycles on dec_r16"),
        }  

        return Status::Completed;
    }

    /**
    * Rotate register A right.
    * 
    * MACHINE CYCLE: 1
    * INSTRUCTION LENGTH: 1
    */
    fn rrca(flag_reg: &mut u8, reg_a: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                Cpu::set_flags(flag_reg, Some(false), Some(false), Some(false), Some(binary_utils::get_bit(*reg_a, 0) != 0));
                *reg_a = (*reg_a).rotate_right(1);
            }
            _ => panic!("1 to many cycles on RLCA"),
        }

        return Status::Completed;
    }

    /**
     * THIS IS VERY SPECIAL NEED TO KNOW MORE ABOUT IT. Helps the gameboy
     * get into a very low power state, but also turns off a lot of peripherals
     */
    fn stop(pc: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                *pc += 1;
            }
            _ => panic!("1 to many cycles on stop"),
        }

        println!("YOU STILL REALLY NEED TO FULLY IMPLEMENT THIS");
        //Need to reset the Timer divider register
        //timer begins ticking again once stop mode ends
        return Status::Completed;
    }

    /**
     * Rotate register A left through carry.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1 
     */
    fn rla(flag_reg: &mut u8, reg_a: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let result = (*reg_a << 1) | Cpu::get_carry_flag(*flag_reg);
                Cpu::set_flags(flag_reg, Some(false), Some(false), Some(false), Some(binary_utils::get_bit(*reg_a, 7) != 0));
                *reg_a = result;
            }
            _ => panic!("1 to many machine cycles in rla")
        }

        return Status::Completed;
    }

    /**
     * Jump by i8 to a different address relative to the pc 
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 2
     */
    fn jr_i8(memory: &Memory, pc: &mut u16, machine_cycle: u8, temp_reg: &mut u16) -> Status {
        let mut status = Status::Running;
        match machine_cycle {
            1 => { 
                *temp_reg = memory.read_byte(*pc) as u16; 
                *pc += 1; 
            },
            2 => {
                let signed_temp_reg = *temp_reg as i8;
                *pc = (*pc).wrapping_add_signed(signed_temp_reg as i16);
                status = Status::Completed;
            }
            _ => panic!("1 to many machine cycles in jr_i8")
        }
        return status;
    }

    /**
     * Rotate register A right through carry.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn rra(flag_reg: &mut u8, reg_a: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let rotated_bit = binary_utils::get_bit(*reg_a, 0);
                *reg_a = (*reg_a >> 1) | (Cpu::get_carry_flag(*flag_reg) << 7);
                Cpu::set_flags(flag_reg, Some(false), Some(false), Some(false), Some(rotated_bit != 0));
            }
            _ => panic!("1 to many machine cycles in rla")
        }
        return Status::Completed;
    }

    /**
     * Relative Jump by i8 if condition cc is met.
     * 
     * MACHINE CYCLES: 3 IF TAKEN/ 2 IF NOT TAKEN
     * INSTRUCTION LENGTH: 2
     */
    fn jr_cc_i8(memory: &Memory, pc: &mut u16, condition: bool, machine_cycle: u8, temp_reg: &mut u16)  -> Status {
        let mut status = Status::Running;
        match machine_cycle {
            1 => { 
                *temp_reg = memory.read_byte(*pc) as u16; 
                *pc += 1; 
                if !condition {
                    status = Status::Completed;
                }
            },
            2 => {
                let signed_temp_reg = *temp_reg as i8;
                *pc = (*pc).wrapping_add_signed(signed_temp_reg as i16);
                status = Status::Completed;
            },
            _ => panic!("1 to many machine cycles in jr_cc_i8"),
        }
        return status;
    }

    /**
     * Store value in register A into the byte pointed by HL and increment HL afterwards.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_hli_a(memory: &mut Memory, reg_a: &mut u8, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let reg_hl = binary_utils::build_16bit_num(*reg_h, *reg_l);
                memory.write_byte(reg_hl, *reg_a);

                let (upper_byte, lower_byte) = split_16bit_num(reg_hl.wrapping_add(1));
                *reg_h = upper_byte;
                *reg_l = lower_byte;
            },
            _ => panic!("1 to many machine cycles in ld_hli_a"),
        }
        return Status::Completed;
    }

    /**
     * Decimal Adjust Accumulator to get a correct BCD representation after an arithmetic instruction.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn daa(flag_reg: &mut u8, reg_a: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let mut carry_flag = Cpu::get_carry_flag(*flag_reg) != 0;
                if Cpu::get_negative_flag(*flag_reg) == 0 {
                    if Cpu::get_carry_flag(*flag_reg) != 0 || *reg_a > 0x99 {
                        *reg_a = (*reg_a).wrapping_add(0x60);
                        carry_flag = true;
                    }
                    if Cpu::get_half_carry_flag(*flag_reg) != 0 || *reg_a & 0x0F > 0x09 {
                        *reg_a = (*reg_a).wrapping_add(0x6); 
                    }
                } else {
                    if Cpu::get_carry_flag(*flag_reg) != 0 {
                        *reg_a = (*reg_a).wrapping_sub(0x60);
                    }

                    if Cpu::get_half_carry_flag(*flag_reg) != 0 {
                        *reg_a = (*reg_a).wrapping_sub(0x6);
                    }
                }
                Cpu::set_flags(flag_reg, Some(*reg_a == 0), None, Some(false), Some(carry_flag));
            },
            _ => panic!("1 to many machine cycles in daa"),
        }
        return Status::Completed;
    }

    /**
     * Load value into register A from the byte pointed by HL and increment HL afterwards.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_a_hli(memory: &Memory, reg_a: &mut u8, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let reg_hl = binary_utils::build_16bit_num(*reg_h, *reg_l);
                *reg_a = memory.read_byte(reg_hl);
        
                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(reg_hl + 1);
                *reg_h = upper_byte;
                *reg_l = lower_byte; 
            },
            _ => panic!("1 to many machine cycles in ld_a_hli"),
        }
        return Status::Completed;                     
    }

    /**
     * Store the complement of the A register into the A register
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn cpl(flag_reg: &mut u8, reg_a: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                *reg_a = !*reg_a;
                Cpu::set_flags(flag_reg, None, Some(true), Some(true), None);
            },
            _ => panic!("1 to many machine cycles in cpl"),
        }
        return Status::Completed;
    }

    /**
     * Load value n16 into register SP.
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 3
     */
    fn ld_sp_u16(memory: &Memory, pc: &mut u16, sp: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => { 
                *sp = memory.read_byte(*pc) as u16;
                *pc += 1;
            },
            2 => {
                *sp |= (memory.read_byte(*pc) as u16) << 8; 
                *pc += 1;
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in ld_sp_u16"),
        }
        return Status::Running;
    }

    /**
     * Store value in register A into the byte pointed by HL and decrement HL afterwards.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_hld_a(memory: &mut Memory, reg_a: &mut u8, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let reg_hl = binary_utils::build_16bit_num(*reg_h, *reg_l);
                memory.write_byte(reg_hl, *reg_a);

                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(reg_hl - 1);
                *reg_h = upper_byte;
                *reg_l = lower_byte;
            },
            _ => panic!("1 to many machine cycles in ld_hld_a"),
        }
        return Status::Completed;
    }

    /**
     * Increment value in register SP by 1. NOT ACCURATE
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn inc_sp(sp: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => *sp = (*sp).wrapping_add(1),
            _ => panic!("1 to many machine cycles in inc_sp"), 
        }
        return Status::Completed;
    }

    /**
     * Increment the byte pointed to by HL by 1.
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 1
     */
    fn inc_hl(flag_reg: &mut u8, memory: &mut Memory, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (), //should be reading from HL here
            2 => {
                let reg_hl = binary_utils::build_16bit_num(*reg_h, *reg_l);
                let mut hl_data = memory.read_byte(reg_hl);
                let test = hl_data;
                hl_data = hl_data.wrapping_add(1);
                memory.write_byte(reg_hl, hl_data);
                Cpu::set_flags(flag_reg, Some(hl_data == 0), Some(false), Some((test & 0xF) + 1 > 0xF), None);

                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in inc_hl"),
        }
        return Status::Running;
    }

    /**
    * Decrement the byte pointed to by HL by 1.
    * 
    * MACHINE CYCLES: 3
    * INSTRUCTION LENGTH: 1
    */
    fn dec_hl(flag_reg: &mut u8, memory: &mut Memory, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (), //read byte at HL. too lazy to implement temp reg at this step just doing it on next step
            2 => {
                let reg_hl = binary_utils::build_16bit_num(*reg_h, *reg_l);
                let hl_data = memory.read_byte(reg_hl).wrapping_sub(1);
                memory.write_byte(reg_hl, hl_data);

                Cpu::set_flags(flag_reg, Some(hl_data == 0), Some(true), Some(hl_data & 0xF == 0xF), None);
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in dec_hl"),
        }
        return Status::Running;
    }

    /**
     * Store value u8 into the byte pointed to by register HL.
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 2
     */
    fn ld_hl_u8(memory: &mut Memory, reg_h: u8, reg_l: u8, pc: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (),    //Should be reading immediate here
            2 => {
                let reg_hl = binary_utils::build_16bit_num(reg_h, reg_l);
                let immediate = memory.read_byte(*pc);
                *pc += 1;
                memory.write_byte(reg_hl, immediate);

                return Status::Completed; 
            },
            _ => panic!("1 to many machine cycles in ld_hl_u8"),
        }
        return Status::Running;
    }

        /**
     * Set the carry flag
     * 
     * MACHINE CYCLE: 1
     * INSTRUCTION LENGTH: 1
     */
    fn scf(flag_reg: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => Cpu::set_flags(flag_reg, None, Some(false), Some(false), Some(true)),
            _ => panic!("1 to many machine cycles in scf"),
        }
        return Status::Completed;
    }

    /**
     * Add the value in sp to hl. Probably not cycle accurate
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn add_hl_sp(flag_reg: &mut u8, reg_h: &mut u8, reg_l: &mut u8, sp: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let reg_hl = binary_utils::build_16bit_num(*reg_h, *reg_l);
                let (result, overflow) = reg_hl.overflowing_add(*sp);
                let half_carry_overflow = (reg_hl & 0x0FFF) + (*sp & 0x0FFF) > 0x0FFF;
                
                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(result);
                *reg_h = upper_byte;
                *reg_l = lower_byte;
        
                Cpu::set_flags(flag_reg, None, Some(false), Some(half_carry_overflow), Some(overflow))
            },
            _ => panic!("1 to many machine cycles in add_hl_sp"),
        }
        return Status::Completed;
    }

    /**
     * Load value into register A from the byte pointed by HL and decrement HL afterwards.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_a_hld(memory: &Memory, reg_a: &mut u8, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let reg_hl = binary_utils::build_16bit_num(*reg_h, *reg_l);
                *reg_a = memory.read_byte(reg_hl);

                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(reg_hl - 1);
                *reg_h = upper_byte;
                *reg_l = lower_byte;
            }
            _ => panic!("1 to many machine cycles in add_hl_sp"),
        }
        return Status::Completed;
    }

 /**
    * Decrement value in register SP by 1.
    * 
    * MACHINE CYCLES: 2
    * INSTRUCTION LENGTH: 1
    */
    pub fn dec_sp(sp: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                *sp = (*sp).wrapping_sub(1);
            }
            _ => panic!("1 to many machine cycles in dec_sp"),
        }
        return Status::Completed;
    }

    /**
    * Complement the carry flag
    */
    fn ccf(flag_reg: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => Cpu::set_flags(flag_reg, None, Some(false), Some(false), Some(Cpu::get_carry_flag(*flag_reg) == 0)),
            _ => panic!("1 to many machine cycles in ccf"), 
        }   
        return Status::Completed;
    }

    /**
     * Load (copy) value in register on the right into register on the left.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn ld_r8_r8(reg_right: u8, reg_left: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => *reg_left = reg_right,
            _ => panic!("1 to many machine cycles in ld_r8_r8"),
        }
        return Status::Completed;
    }

    /**
     * Load value into register r8 from the byte pointed to by register HL.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_r8_hl(memory: &Memory, reg_h: u8, reg_l: u8, reg: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => *reg = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l)),
            _ => panic!("1 to many machine cycles in ld_r8_hl"),
        }
        return Status::Completed;
    }

        /**
     * Store value in register r8 into the byte pointed to by register HL.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_hl_r8(memory: &mut Memory, reg: u8, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => memory.write_byte(binary_utils::build_16bit_num(reg_h, reg_l), reg),
            _ => panic!("1 to many machine cycles in ld_hl_r8"),
        }
        return Status::Completed;
    }

    /**
     * Enter CPU low-power consumption mode until an interrupt occurs. 
     * The exact behavior of this instruction depends on the state of the IME flag.
     * 
     * MACHINE CYCLES: -
     * INSTRUCTION LENGTH: 1
     */
    fn halt(&mut self) -> Status {
        return Status::Completed;
    }

    /**
     * Add the value in r8 to A.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1 
     */
    fn add_a_r8(flag_reg: &mut u8, reg: u8, reg_a: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let (result, overflow) = (*reg_a).overflowing_add(reg); 
                let half_carry_overflow = (*reg_a & 0xF) + (reg & 0xF) > 0xF;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(half_carry_overflow), Some(overflow));
            },
            _ => panic!("1 to many machine cycles in add_a_r8"),
        }
        return Status::Completed;
    }

    /**
    * Add the byte pointed to by HL to A.
    * 
    * MACHINE CYCLES: 2
    * INSTRUCTION LENGTH: 1 
    */
    fn add_a_hl(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                let (result, overflow) = (*reg_a).overflowing_add(hl_data);
                let half_carry_overflow = (*reg_a & 0xF) + (hl_data & 0xF) > 0xF;
        
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(half_carry_overflow), Some(overflow));
            },
            _ => panic!("1 to many machine cycles in add_a_hl"),
        }
        return Status::Completed;
    }

    /**
     * Add the value in r8 plus the carry flag to A.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1 
     */
    fn adc_a_r8(flag_reg: &mut u8, reg: u8, reg_a: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let (partial_result, first_overflow) = (*reg_a).overflowing_add(reg);
                let (result, second_overflow) = partial_result.overflowing_add(Cpu::get_carry_flag(*flag_reg));
                let half_carry_overflow = (*reg_a & 0xF) + (reg & 0xF) + Cpu::get_carry_flag(*flag_reg) > 0xF;
        
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(half_carry_overflow), Some(first_overflow || second_overflow));
            },
            _ => panic!("1 to many machine cycles in adc_a_r8"),
        }
        return Status::Completed;
    }

    /**
     * Add the byte pointed to by HL plus the carry flag to A.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1 
     */
    fn adc_a_hl(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                let (partial_result, first_overflow) = (*reg_a).overflowing_add(hl_data);
                let (result, second_overflow) = partial_result.overflowing_add(Cpu::get_carry_flag(*flag_reg));
                let half_carry_overflow = (*reg_a & 0xF) + (hl_data & 0xF) + Cpu::get_carry_flag(*flag_reg) > 0xF;
        
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(half_carry_overflow), Some(first_overflow || second_overflow));
            },
            _ => panic!("1 to many machine cycles in adc_a_hl"),
        }
        return Status::Completed;
    }

    /**
    * Subtract the value in r8 from A.
    * 
    * MACHINE CYCLES: 1
    * INSTRUCTION LENGTH: 1
    */
    fn sub_a_r8(flag_reg: &mut u8, reg: u8, reg_a: &mut u8, machine_cycle: u8) -> Status { //FIX THIS
        match machine_cycle {
            1 => {
                let result = (*reg_a).wrapping_sub(reg);
                let half_carry = (*reg_a & 0xF).wrapping_sub(reg & 0xF) > 0xF;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some(half_carry), Some(reg > *reg_a));
                *reg_a = result;
            },
            _ => panic!("1 to many machine cycles in sub_a_r8"),
        }
        return Status::Completed;
    }

    /**
     * Subtract the byte pointed to by HL from A.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn sub_a_hl(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                let result = (*reg_a).wrapping_sub(hl_data);
                let half_carry = (((*reg_a & 0xf).wrapping_sub(hl_data & 0xf)) & 0x10) == 0x10;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some(half_carry), Some(hl_data > *reg_a));

                *reg_a = result;
            },
            _ => panic!("1 to many machine cycles in sub_a_hl"),
        }
        return Status::Completed;
    }

    /**
     * Subtract the value in r8 and the carry flag from A.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn sbc_a_r8(flag_reg: &mut u8, reg: u8, reg_a: &mut u8 ,machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let r1 = (*reg_a).wrapping_sub(Cpu::get_carry_flag(*flag_reg));
                let result = r1.wrapping_sub(reg);

                let half_carry_1 = (((*reg_a & 0xf).wrapping_sub(Cpu::get_carry_flag(*flag_reg) & 0xf)) & 0x10) == 0x10;
                let half_carry_2 = (((r1 & 0xf).wrapping_sub(reg & 0xf)) & 0x10) == 0x10;
                let carry_1 = Cpu::get_carry_flag(*flag_reg) > *reg_a;
                let carry_2 = reg > r1;

                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some(half_carry_1 || half_carry_2), Some(carry_1 || carry_2));
                *reg_a = result;
            },
            _ => panic!("1 to many machine cycles in sbc_a_r8"),
        }
        return Status::Completed;
    } 

    /**
     * Subtract the byte pointed to by HL and the carry flag from A.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn sbc_a_hl(flag_reg: &mut u8, reg_h: u8, reg_l: u8, reg_a: &mut u8, memory: &Memory, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                let pre_result = (*reg_a).wrapping_sub(Cpu::get_carry_flag(*flag_reg));
                let result = (pre_result).wrapping_sub(hl_data);

                let half_carry_1 = (((*reg_a & 0xf).wrapping_sub(pre_result & 0xf)) & 0x10) == 0x10;
                let half_carry_2 = (((pre_result & 0xf).wrapping_sub(hl_data & 0xf)) & 0x10) == 0x10;
                let carry_1 = Cpu::get_carry_flag(*flag_reg) > *reg_a;
                let carry_2 = hl_data > pre_result;
        
                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some(half_carry_1 || half_carry_2), Some(carry_1 || carry_2));
                
                *reg_a = result;
            },
            _ => panic!("1 to many machine cycles in sbc_a_r8"),
        }
        return Status::Completed;
    }

/**
     * Bitwise AND between the value in r8 and A.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn and_a_r8(flag_reg: &mut u8, reg: u8, reg_a: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let result = reg & *reg_a;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(true), Some(false));
            } ,
            _ => panic!("1 to many machine cycles in and_a_r8"),
        }
        return Status::Completed;
    }

    /**
     * Bitwise AND between the byte pointed to by HL and A.
     * 
     * MACHINE CYCLE: 2
     * INSTRUCTION LENGTH: 1
     */
    fn and_a_hl(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle { 
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                let result = hl_data & *reg_a;
        
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(true), Some(false));
            },
            _ => panic!("1 to many machine cycles in and_a_hl"),
        }
        return Status::Completed;
    }

    /**
     * Bitwise XOR between the value in r8 and A.
     * 
     * MACHINE CYCLE: 1
     * INSTRUCTION LENGTH: 1
     */
    fn xor_a_r8(flag_reg: &mut u8, reg: u8, reg_a: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let result = reg ^ *reg_a;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(false));
            },
            _ => panic!("1 to many machine cycles in xor_a_r8"),
        }
        return Status::Completed;
    }

    /**
     * Bitwise XOR between the byte pointed to by HL and A.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn xor_a_hl(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                *reg_a = hl_data ^ *reg_a;
                Cpu::set_flags(flag_reg, Some(*reg_a == 0), Some(false), Some(false), Some(false));
            }
            _ => panic!("1 to many machine cycles in xor_a_hl"),
        }
        return Status::Completed;
    }

    /**
     * Bitwise OR between the value in r8 and A. 
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn or_a_r8(flag_reg: &mut u8, reg: u8, reg_a: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let result = reg | *reg_a;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(false));
            }, 
            _ => panic!("1 to many machine cycles in or_a_r8"),
        }
        return Status::Completed;

    }

    /**
    * Bitwise OR between the byte pointed to by HL and A.
    * 
    * MACHINE CYCLES: 2
    * INSTRUCTION LENGTH: 1
    */
    fn or_a_hl(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, reg_h: u8, reg_l: u8, machine_cycle: u8)  -> Status {
        match machine_cycle {
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                let result = hl_data | *reg_a;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(false));
            },
            _ => panic!("1 to many machine cycles in or_a_hl"),
        }
        return Status::Completed;
    } 

    /**
     * Subtract the value in r8 from A and set flags accordingly, but don't store the result. This is useful for ComParing values.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1    
     */
    fn cp_a_r8(flag_reg: &mut u8, reg: u8, reg_a: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let result = reg_a.wrapping_sub(reg);
                let half_carry = (reg_a & 0xF).wrapping_sub(reg & 0xF) > 0xF;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some(half_carry), Some(reg > reg_a));
            },
            _ => panic!("1 to many machine cycles in cp_a_r8"),
        }
        return Status::Completed;
    }

    /**
     * Subtract the byte pointed to by HL from A and set flags accordingly, but don't store the result.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn cp_a_hl(flag_reg: &mut u8, memory: &Memory, reg_a: u8, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                let result = reg_a.wrapping_sub(hl_data);
                let half_carry = (((reg_a & 0xf).wrapping_sub(hl_data & 0xf)) & 0x10) == 0x10;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some(half_carry), Some(hl_data > reg_a));
            },
            _ => panic!("1 to many machine cycles in cp_a_hl"),
        }
        return Status::Completed;
    }

    /**
     * Return from subroutine if condition cc is met.
     * 
     * MACHINE CYCLES: 5 IF TAKEN/ 2 IF NOT TAKEN
     * INSTRUCTION LENGTH: 1
     */
    fn ret_cc(memory: &mut Memory, sp: &mut u16, pc: &mut u16, condition: bool, machine_cycle: u8, temp_reg: &mut u16) -> Status {
        match machine_cycle {
            1 => {
                if !condition {
                    return Status::Completed;
                }
            },
            2 => { *temp_reg = memory.read_byte(*sp) as u16; *sp += 1; },         //Read lower SP byte
            3 => { *temp_reg |= (memory.read_byte(*sp) as u16) << 8; *sp += 1; }, //Read upper SP byte
            4 =>  {
                *pc = *temp_reg;
                memory.interrupt_handler.handling_interrupt = Interrupt::Idle;
                return Status::Completed;
            }
            _ => panic!("1 to many machine cycles in ret_cc"),
        }
        return Status::Running;
    }

    /**
     * Pop to register r16 from the stack.
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 1
     */
    fn pop(memory: &Memory, upper_reg: &mut u8, lower_reg: &mut u8, sp: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => { 
                *lower_reg = memory.read_byte(*sp); 
                *sp += 1; 
            },
            2 => { 
                *upper_reg = memory.read_byte(*sp); 
                *sp += 1; 
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in pop"),
        }
        return Status::Running;
    }

    /**
    * Jump to address u16 if the condition is met
    * 
    * MACHINE CYCLES: 4
    * INSTRUCTION LENGTH: 3
    */
    fn jp_cc_u16(memory: &Memory, pc: &mut u16, condition: bool, machine_cycle: u8, temp_reg: &mut u16) -> Status {
        match machine_cycle {
            1 => {
                *temp_reg = memory.read_byte(*pc) as u16;
                *pc += 1;
            },
            2 => {
                *temp_reg |= (memory.read_byte(*pc) as u16) << 8;
                *pc += 1;

                if !condition {
                    return Status::Completed;
                }
            },
            3 => {
                *pc = *temp_reg;
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in jp_cc_u16"),
        }
        return Status::Running;
    }

    /**
    * Jump to address u16
    * 
    * MACHINE CYCLES: 4
    * INSTRUCTION LENGTH: 3
    */
    fn jp_u16(memory: &Memory, pc: &mut u16, machine_cycle: u8, temp_reg: &mut u16) -> Status {
        match machine_cycle {
            1 => { 
                *temp_reg = memory.read_byte(*pc) as u16;
                *pc += 1;
            }, 
            2 => {
                *temp_reg |= (memory.read_byte(*pc) as u16) << 8;
                *pc += 1;
            },
            3 => {
                *pc = *temp_reg;
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in jp_u16"),
        }   

        return Status::Running;
    }

    /**
     * Call address u16 if condition cc is met.
     * 
     * MACHINE CYCLES: 6 IF TAKEN/ 3 IF NOT TAKEN
     * INSTRUCTION LENGTH: 3
     */
    fn call_cc_u16(memory: &mut Memory, pc: &mut u16, sp: &mut u16, condition: bool, machine_cycle: u8, temp_reg: &mut u16) -> Status {
        match machine_cycle {
            1 => {
                *temp_reg = memory.read_byte(*pc) as u16;
                *pc += 1;
            },
            2 => {
                *temp_reg |= (memory.read_byte(*pc) as u16) << 8;
                *pc += 1;

                if !condition {
                    return Status::Completed;
                }
            },
            3 => {
                *sp -= 1;
                let (upper_byte, _) = binary_utils::split_16bit_num(*pc); 
                memory.write_byte(*sp, upper_byte);
            },
            4 => {
                *sp -= 1;
                let (_, lower_byte) = binary_utils::split_16bit_num(*pc); 
                memory.write_byte(*sp, lower_byte);
            },
            5 => {
                *pc = *temp_reg;
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in call_cc_u16"),
        }     
        return Status::Running;
    }

    /**
     * Push register r16 into the stack
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 1
     */
    fn push_r16(memory: &mut Memory, upper_reg: u8, lower_reg: u8, sp: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (), //No clue that this is doing at this cycle
            2 => {
                *sp -= 1;
                memory.write_byte(*sp, upper_reg);
            },
            3 => {
                *sp -= 1;
                memory.write_byte(*sp, lower_reg);
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in push_r16"),
        }
        return Status::Running;
    }

    /**
     * Add the value u8 to A.
     * 
     * MACHINE CYCLE: 2
     * INSTRUCTION LENGTH: 2
     */
    fn add_a_u8(flag_reg: &mut u8, memory: &Memory, pc: &mut u16, reg_a: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let value = memory.read_byte(*pc);
                *pc += 1;
        
                let (result, overflow) = (*reg_a).overflowing_add(value);
                let half_carry_overflow = (*reg_a & 0xF) + (value & 0xF) > 0xF;
        
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(half_carry_overflow), Some(overflow));
            }, 
            _ => panic!("1 to many machine cycles in add_a_u8"),
        }
        return Status::Completed;
    }

    /**
     * Call address vec. This is a shorter and faster equivalent to CALL for suitable values of vec. Possibly not cycle accurate
     * 
     * MACHINE CYCLE: 4
     * INSTRUCTION LENGTH: 1
     */
    fn rst_vec(memory: &mut Memory, sp: &mut u16, pc: &mut u16, rst_address: u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                *sp = (*sp).wrapping_sub(1);
                let (upper_byte, _) = binary_utils::split_16bit_num(*pc);
                memory.write_byte(*sp, upper_byte);
            },
            2 => {
                *sp = (*sp).wrapping_sub(1);
                let (_, lower_byte) = binary_utils::split_16bit_num(*pc);
                memory.write_byte(*sp, lower_byte);
            },
            3 =>{
                *pc = rst_address;
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in rst_vec"),
        }
        return Status::Running;
    }

    /**
     * Return from subroutine. This is basically a POP PC (if such an instruction existed). See POP r16 for an explanation of how POP works.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 1
     */
    fn ret(memory: &mut Memory, sp: &mut u16, pc: &mut u16, machine_cycle: u8, temp_reg: &mut u16) -> Status {
        match machine_cycle {
            1 => {
                *temp_reg = memory.read_byte(*sp) as u16;
                *sp += 1;
            },
            2 =>{
                *temp_reg |= (memory.read_byte(*sp) as u16) << 8;
                *sp += 1;
            }
            3 =>{
                *pc = *temp_reg;
                memory.interrupt_handler.handling_interrupt = Interrupt::Idle;
                return Status::Completed;
            }
            _ => panic!("1 to many machine cycles in ret"),
        }
        return Status::Running;
    }

    /**
    * Call address n16. This pushes the address of the instruction after the CALL on the stack, 
    * such that RET can pop it later; then, it executes an implicit JP n16.
    * 
    * MACHINE CYCLES: 6
    * INSTRUCTION LENGTH: 3
    */
    fn call_u16(memory: &mut Memory, pc: &mut u16, sp: &mut u16, machine_cycle: u8, temp_reg: &mut u16) -> Status {
        match machine_cycle {
            1 => {
                *temp_reg = memory.read_byte(*pc) as u16;
                *pc += 1;
            },
            2 => {
                *temp_reg |= (memory.read_byte(*pc) as u16) << 8;
                *pc += 1;
            },
            3 => {
                *sp -= 1;
                let (upper_byte, _) = binary_utils::split_16bit_num(*pc);
                memory.write_byte(*sp, upper_byte);
            },
            4 => {
                *sp -= 1;
                let (_, lower_byte) = binary_utils::split_16bit_num(*pc);
                memory.write_byte(*sp, lower_byte);
            },
            5 => {
                *pc = *temp_reg;
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in call_u16"),
        }
        return Status::Running;
    }

    /**
     * Add the value u8 plus the carry flag to A.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2 
     */
    fn adc_a_u8(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, pc: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let value = memory.read_byte(*pc);
                *pc += 1;
        
                let (partial_result, first_overflow) = (*reg_a).overflowing_add(value);
                let (result, second_overflow) = partial_result.overflowing_add(Cpu::get_carry_flag(*flag_reg));
                let half_carry_overflow = (*reg_a & 0xF) + (value & 0xF) + Cpu::get_carry_flag(*flag_reg) > 0xF;
        
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(half_carry_overflow), Some(first_overflow || second_overflow));
            },
            _ => panic!("1 to many machine cycles in adc_a_u8"),
        }
        return Status::Completed;
    }

    /**
    * Subtract the value u8 from A.
    * 
    * MACHINE CYCLES: 2
    * INSTRUCTION LENGTH: 2
    */
    fn sub_a_u8(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, pc: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let value = memory.read_byte(*pc);
                *pc += 1;

                let half_carry = (((*reg_a & 0xf).wrapping_sub(value & 0xf)) & 0x10) == 0x10;
                let result = (*reg_a).wrapping_sub(value);
                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some(half_carry), Some(value > *reg_a));
                *reg_a = result;
            },
            _ => panic!("1 to many machine cycles in sub_a_u8"),
        }
        return Status::Completed;
    }

    /**
     * Return from subroutine and enable interrupts. 
     * This is basically equivalent to executing EI then RET, meaning that IME is set right after this instruction.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 1
     */
    fn reti(memory: &mut Memory, sp: &mut u16, pc: &mut u16, machine_cycle: u8, temp_reg: &mut u16) -> Status {
        match machine_cycle {
            1 => {
                *temp_reg = memory.read_byte(*sp) as u16;
                *sp += 1;
            },
            2 => {
                *temp_reg |= (memory.read_byte(*sp) as u16) << 8;
                *sp += 1;
            },
            3 => {
                *pc = *temp_reg;
                memory.interrupt_handler.enable_ime_flag();
                memory.interrupt_handler.handling_interrupt = interrupt_handler::Interrupt::Idle;
                return Status::Completed;
            }
            _ => panic!("1 to many machine cycles in reti"),
        }
        return Status::Running;
        //todo!("Need to come back to this instruction as we need to delay the EI flag by 1 machine cycle. They say this is like EI then RET so I might just call those 2 functions");
    }

    fn sbc_a_u8(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, pc: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let value = memory.read_byte(*pc);
                *pc += 1;
                
                let r1 = (*reg_a).wrapping_sub(Cpu::get_carry_flag(*flag_reg));
                let result = r1.wrapping_sub(value);

                let half_carry_1 = (((*reg_a & 0xf).wrapping_sub(Cpu::get_carry_flag(*flag_reg) & 0xf)) & 0x10) == 0x10;
                let half_carry_2 = (((r1 & 0xf).wrapping_sub(value & 0xf)) & 0x10) == 0x10;
                let carry_1 = Cpu::get_carry_flag(*flag_reg) > *reg_a;
                let carry_2 = value > r1;

                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some(half_carry_1 || half_carry_2), Some(carry_1 || carry_2));
                *reg_a = result;
            },
            _ => panic!("1 to many machine cycles in sbc_a_u8"),
        }
        return Status::Completed;
    }

    /**
     * Store value in register A into the byte at address $FF00+C.
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 2
     */
    fn ldh_u8_a(memory: &mut Memory, pc: &mut u16, reg_a: u8, machine_cycle: u8, temp_reg: &mut u16) -> Status {
        match machine_cycle {
            1 => {
                *temp_reg = 0xFF00 | (memory.read_byte(*pc) as u16);
                *pc += 1;
            }
            2 => {
                memory.write_byte(*temp_reg, reg_a);
                return Status::Completed;
            }
            _ => panic!("1 to many machine cycles in ldh_u8_a"),
        }
        return Status::Running;
    }

    /**
    * Store value in register A into the byte at address $FF00+C.
    * 
    * MACHINE CYCLES: 2
    * INSTRUCTION LENGTH: 1
    */
    fn ldh_c_a(memory: &mut Memory, reg_a: u8, reg_c: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                memory.write_byte(0xFF00 | reg_c as u16, reg_a);
            },
            _ => panic!("1 to many machine cycles in ldh_c_a"),
        }
        return Status::Completed;
    }

    /**
     * Bitwise AND between u8 immediate and 8-bit A register
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION CYCLES: 2
     */
    fn and_a_u8(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, pc: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let value = memory.read_byte(*pc);
                *pc += 1;
        
                let result = value & *reg_a;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(true), Some(false));
            },
            _ => panic!("1 to many machine cycles in and_a_u8"),
        }
        return Status::Completed;
    }

    /**
     * Add the 8-bit signed value i8 to 16-bit SP register.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 2
     */
    fn add_sp_i8(flag_reg: &mut u8, memory: &Memory,sp: &mut u16, pc: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle { 
            1 => (), //Would read the i8 operand
            2 => (), //Would write the SP lower byte
            3 => {
                let value_i8 = memory.read_byte(*pc) as i8;
                *pc += 1;
                
                let result: u16 = (*sp).wrapping_add_signed(value_i8 as i16);

                let sp_u8 = *sp as u8;
                let half_carry = (sp_u8 & 0xF).wrapping_add((value_i8 as u8) & 0xF) > 0xF;
                let (_, carry) = sp_u8.overflowing_add(value_i8 as u8);


                // if value_i8 < 0 {
                //     let x = value_i8.abs() as u8;
                //     half_carry = (((*sp & 0xf).wrapping_sub((x as u16) & 0xf)) & 0x10) == 0x10;
                //     carry_overflow = (*sp & 0xFF).wrapping_sub((x as u16) & 0xFF) > 0xFF;
                // } else {
                //     let x = value_i8 as u8;
                //     half_carry = (*sp & 0xF).wrapping_add((x as u16) & 0xF) > 0xF;
                //     carry_overflow = (*sp & 0xFF).wrapping_add((x as u16) & 0xFF) > 0xFF;
                // }
        
                *sp = result;
                Cpu::set_flags(flag_reg, Some(false), Some(false), Some(half_carry), Some(carry));
                //Cpu::set_flags(flag_reg, Some(false), Some(false), Some(half_carry), Some(value > *reg_a));
                return Status::Completed;
            }
            
            _ => panic!("1 to many machine cycles in add_sp_i8"),
        }
        return Status::Running;
    }

    /**
     * Jump to address in HL; effectively, load PC with value in register HL.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn jp_hl(reg_h: u8, reg_l: u8, pc: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                *pc = binary_utils::build_16bit_num(reg_h, reg_l);
            },
            _ => panic!("1 to many machine cycles in jp_hl"),
        }
        return Status::Completed;
    }

    
    /**
     * Store value in register A into the byte at address u16.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 3
     */
    fn ld_u16_a(memory: &mut Memory, pc: &mut u16, reg_a: u8, machine_cycle: u8, temp_reg: &mut u16) -> Status {
        match machine_cycle {
            1 => {
                *temp_reg = memory.read_byte(*pc) as u16;
                *pc += 1;
            },
            2 => {
                *temp_reg |= (memory.read_byte(*pc) as u16) << 8;
                *pc += 1;
            },
            3 => {
                memory.write_byte(*temp_reg, reg_a);
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in ld_u16_a"),
        }
        return Status::Running;
    }

    /**
     * Bitwise XOR between the value in u8 and A.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn xor_a_u8(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, pc: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let value = memory.read_byte(*pc);
                *pc += 1;
        
                *reg_a = value ^ *reg_a;
                Cpu::set_flags(flag_reg, Some(*reg_a == 0), Some(false), Some(false), Some(false));
            },
            _ => panic!("1 to many machine cycles in xor_a_u8"),
        }
        return Status::Completed;
    }

    fn ldh_a_u8(memory: &Memory, pc: &mut u16, reg_a: &mut u8, machine_cycle: u8, temp_reg: &mut u16) -> Status {
        match machine_cycle {
            1 => {
                *temp_reg = 0xFF00 | (memory.read_byte(*pc) as u16);
                *pc += 1;
            },
            2 => {
                *reg_a = memory.read_byte(*temp_reg);
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in ldh_a_u8"),
        }
        return Status::Running;
    }

    /**
     * Load value in register A from the byte at address $FF00+C.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ldh_a_c(memory: &Memory, reg_a: &mut u8, reg_c: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                *reg_a = memory.read_byte(0xFF00 | reg_c as u16);
            },
            _ => panic!("1 to many machine cycles in ldh_a_c"),
        }
        return Status::Completed;
    }

    /**
     * Disable Interrupts by clearing the IME flag. One reading mentions it cancels any scheduled effects of EI instruction
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn di(memory: &mut Memory, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                memory.interrupt_handler.disable_ime_flag();
            },
            _ => panic!("1 to many machine cycles in di"),
        }
        return Status::Completed;
    }

    /**
     * Store into A the bitwise OR of u8 and A.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn or_a_u8(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, pc: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let value = memory.read_byte(*pc);
                *pc += 1;
        
                let result = value | *reg_a;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(false));
            },
            _ => panic!("1 to many machine cycles in or_a_u8"),
        }
        return Status::Completed;
    }

    /**
     * Add the signed value i8 to SP and store the result in HL. CARRY HERE IS SO WRONG
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 2
     */
    fn ld_hl_sp_i8(flag_reg: &mut u8, memory: &Memory, sp: &mut u16, pc: &mut u16, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (), //Should be reading from i8 operand
            2 => {
                let value = memory.read_byte(*pc) as i8;
                *pc += 1;

                let result: u16 = (*sp).wrapping_add_signed(value as i16);
                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(result);
                *reg_h = upper_byte;
                *reg_l = lower_byte;

                let sp_u8 = *sp as u8;
                let half_carry = (sp_u8 & 0xF).wrapping_add((value as u8) & 0xF) > 0xF;
                let (_, carry) = sp_u8.overflowing_add(value as u8);
        
                Cpu::set_flags(flag_reg, Some(false), Some(false), Some(half_carry), Some(carry));
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in ld_hl_sp_i8"),
        }
        return Status::Running;
    }

    /**
     * Load register HL into register SP.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_sp_hl(reg_h: u8, reg_l: u8, sp: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                *sp = binary_utils::build_16bit_num(reg_h, reg_l);
            },
            _ => panic!("1 to many machine cycles in ld_sp_hl"),
        }
        return Status::Completed;
    }

    /**
     * Load value in register A from the byte at address u16.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 3
     */
    fn ld_a_u16(memory: &Memory, pc: &mut u16, reg_a: &mut u8, machine_cycle: u8, temp_reg: &mut u16) -> Status {
        match machine_cycle {
            1 => {
                *temp_reg = memory.read_byte(*pc) as u16;
                *pc += 1;
            },
            2 => {
                *temp_reg |= (memory.read_byte(*pc) as u16) << 8;
                *pc += 1;
            },
            3 => {
                *reg_a = memory.read_byte(*temp_reg);
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in ld_a_u16"),
        }
        return Status::Running;
    }

    /**
     * Enable Interrupts by setting the IME flag. The flag is only set after the instruction following EI.\
     * 
     * MACHINE CYCLE: 1
     * INSTRUCTION LENGTH: 1
     */
    fn ei(memory: &mut Memory, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                memory.interrupt_handler.enable_ime_flag();
            },
            _ => panic!("1 to many machine cycles in ei"),
        }
        return Status::Completed;
    }

    /**
     * Subtract the value u8 from A and set flags accordingly, but don't store the result.
     * 
     * MACHINE CYCLE: 2
     * INSTRUCTION LENGTH: 2
     */
    fn cp_a_u8(flag_reg: &mut u8, memory: &Memory, reg_a: u8, pc: &mut u16, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let value = memory.read_byte(*pc);
                *pc += 1;
        
                let result = reg_a.wrapping_sub(value);
                let half_carry = (((reg_a & 0xf).wrapping_sub(value & 0xf)) & 0x10) == 0x10;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some(half_carry), Some(value > reg_a));
            },
            _ => panic!("1 to many machine cycles in cp_a_u8"),
        }
        return Status::Completed;
    }

    /**
     * This will execute the instruction of the opcode on the prefix table. 
     * If the instruction is not complete it will return a Running status.
     */
    fn exexute_prefix(&mut self, memory: &mut Memory, machine_cycle: u8) -> Status {
        match self.current_opcode {
            0x00 => Cpu::rlc_r8(&mut self.f, &mut self.b, machine_cycle),                   //RLC B    
            0x01 => Cpu::rlc_r8(&mut self.f, &mut self.c, machine_cycle),                   //RLC C 
            0x02 => Cpu::rlc_r8(&mut self.f, &mut self.d, machine_cycle),                   //RLC D 
            0x03 => Cpu::rlc_r8(&mut self.f, &mut self.e, machine_cycle),                   //RLC E 
            0x04 => Cpu::rlc_r8(&mut self.f, &mut self.h, machine_cycle),                   //RLC H 
            0x05 => Cpu::rlc_r8(&mut self.f, &mut self.l, machine_cycle),                   //RLC L
            0x06 => Cpu::rlc_hl(&mut self.f, memory, self.h, self.l, machine_cycle),        //RLC (HL)
            0x07 => Cpu::rlc_r8(&mut self.f, &mut self.a, machine_cycle),                   //RLC A
            0x08 => Cpu::rrc_r8(&mut self.f, &mut self.b, machine_cycle),                   //RRC B
            0x09 => Cpu::rrc_r8(&mut self.f, &mut self.c, machine_cycle),                   //RRC C
            0x0A => Cpu::rrc_r8(&mut self.f, &mut self.d, machine_cycle),                   //RRC D
            0x0B => Cpu::rrc_r8(&mut self.f, &mut self.e, machine_cycle),                   //RRC E
            0x0C => Cpu::rrc_r8(&mut self.f, &mut self.h, machine_cycle),                   //RRC H
            0x0D => Cpu::rrc_r8(&mut self.f, &mut self.l, machine_cycle),                   //RRC L
            0x0E => Cpu::rrc_hl(&mut self.f, memory, self.h, self.l, machine_cycle),        //RRC (HL)
            0x0F => Cpu::rrc_r8(&mut self.f, &mut self.a, machine_cycle),                   //RRC A
            0x10 => Cpu::rl_r8(&mut self.f, &mut self.b, machine_cycle),                    //RL B
            0x11 => Cpu::rl_r8(&mut self.f, &mut self.c, machine_cycle),                    //RL C
            0x12 => Cpu::rl_r8(&mut self.f, &mut self.d, machine_cycle),                    //RL D
            0x13 => Cpu::rl_r8(&mut self.f, &mut self.e, machine_cycle),                    //RL E
            0x14 => Cpu::rl_r8(&mut self.f, &mut self.h, machine_cycle),                    //RL H
            0x15 => Cpu::rl_r8(&mut self.f, &mut self.l, machine_cycle),                    //RL L
            0x16 => Cpu::rl_hl(&mut self.f, memory, self.h, self.l, machine_cycle),         //RL (HL)
            0x17 => Cpu::rl_r8(&mut self.f, &mut self.a, machine_cycle),                    //RL A
            0x18 => Cpu::rr_r8(&mut self.f, &mut self.b, machine_cycle),                    //RR B
            0x19 => Cpu::rr_r8(&mut self.f, &mut self.c, machine_cycle),                    //RR C
            0x1A => Cpu::rr_r8(&mut self.f, &mut self.d, machine_cycle),                    //RR D
            0x1B => Cpu::rr_r8(&mut self.f, &mut self.e, machine_cycle),                    //RR E
            0x1C => Cpu::rr_r8(&mut self.f, &mut self.h, machine_cycle),                    //RR H
            0x1D => Cpu::rr_r8(&mut self.f, &mut self.l, machine_cycle),                    //RR L
            0x1E => Cpu::rr_hl(&mut self.f, memory, self.h, self.l, machine_cycle),         //RR (HL)
            0x1F => Cpu::rr_r8(&mut self.f, &mut self.a, machine_cycle),                    //RR A
            0x20 => Cpu::sla_r8(&mut self.f, &mut self.b, machine_cycle),                   //SLA B
            0x21 => Cpu::sla_r8(&mut self.f, &mut self.c, machine_cycle),                   //SLA C
            0x22 => Cpu::sla_r8(&mut self.f, &mut self.d, machine_cycle),                   //SLA D
            0x23 => Cpu::sla_r8(&mut self.f, &mut self.e, machine_cycle),                   //SLA E
            0x24 => Cpu::sla_r8(&mut self.f, &mut self.h, machine_cycle),                   //SLA H
            0x25 => Cpu::sla_r8(&mut self.f, &mut self.l, machine_cycle),                   //SLA L
            0x26 => Cpu::sla_hl(&mut self.f, memory, self.h, self.l, machine_cycle),        //SLA (HL)
            0x27 => Cpu::sla_r8(&mut self.f, &mut self.a, machine_cycle),                   //SLA A
            0x28 => Cpu::sra_r8(&mut self.f, &mut self.b, machine_cycle),                   //SRA B
            0x29 => Cpu::sra_r8(&mut self.f, &mut self.c, machine_cycle),                   //SRA C
            0x2A => Cpu::sra_r8(&mut self.f, &mut self.d, machine_cycle),                   //SRA D
            0x2B => Cpu::sra_r8(&mut self.f, &mut self.e, machine_cycle),                   //SRA E
            0x2C => Cpu::sra_r8(&mut self.f, &mut self.h, machine_cycle),                   //SRA H
            0x2D => Cpu::sra_r8(&mut self.f, &mut self.l, machine_cycle),                   //SRA L
            0x2E => Cpu::sra_hl(&mut self.f, memory, self.h, self.l, machine_cycle),        //SRA (HL)
            0x2F => Cpu::sra_r8(&mut self.f, &mut self.a, machine_cycle),                   //SRA A
            0x30 => Cpu::swap_r8(&mut self.f, &mut self.b, machine_cycle),                  //SWAP B
            0x31 => Cpu::swap_r8(&mut self.f, &mut self.c, machine_cycle),                  //SWAP C
            0x32 => Cpu::swap_r8(&mut self.f, &mut self.d, machine_cycle),                  //SWAP D
            0x33 => Cpu::swap_r8(&mut self.f, &mut self.e, machine_cycle),                  //SWAP E
            0x34 => Cpu::swap_r8(&mut self.f, &mut self.h, machine_cycle),                  //SWAP H
            0x35 => Cpu::swap_r8(&mut self.f, &mut self.l, machine_cycle),                  //SWAP L
            0x36 => Cpu::swap_hl(&mut self.f, memory, self.h, self.l, machine_cycle),       //SWAP (HL)
            0x37 => Cpu::swap_r8(&mut self.f, &mut self.a, machine_cycle),                  //SWAP A
            0x38 => Cpu::srl_r8(&mut self.f, &mut self.b, machine_cycle),               //SRL B
            0x39 => Cpu::srl_r8(&mut self.f, &mut self.c, machine_cycle),               //SRL C
            0x3A => Cpu::srl_r8(&mut self.f, &mut self.d, machine_cycle),               //SRL D
            0x3B => Cpu::srl_r8(&mut self.f, &mut self.e, machine_cycle),               //SRL E
            0x3C => Cpu::srl_r8(&mut self.f, &mut self.h, machine_cycle),               //SRL H
            0x3D => Cpu::srl_r8(&mut self.f, &mut self.l, machine_cycle),               //SRL L
            0x3E => Cpu::srl_hl(&mut self.f, memory, self.h, self.l, machine_cycle),    //SRL (HL)
            0x3F => Cpu::srl_r8(&mut self.f, &mut self.a, machine_cycle),               //SRL A
            0x40 => Cpu::bit_u3_r8(&mut self.f, 0, self.b, machine_cycle),              //BIT 0, B
            0x41 => Cpu::bit_u3_r8(&mut self.f, 0, self.c, machine_cycle),              //BIT 0, C
            0x42 => Cpu::bit_u3_r8(&mut self.f, 0, self.d, machine_cycle),              //BIT 0, D
            0x43 => Cpu::bit_u3_r8(&mut self.f, 0, self.e, machine_cycle),              //BIT 0, E
            0x44 => Cpu::bit_u3_r8(&mut self.f, 0, self.h, machine_cycle),              //BIT 0, H
            0x45 => Cpu::bit_u3_r8(&mut self.f, 0, self.l, machine_cycle),              //BIT 0, L
            0x46 => Cpu::bit_u3_hl(&mut self.f, memory, self.h, self.l, 0, machine_cycle),   //BIT 0, (HL)
            0x47 => Cpu::bit_u3_r8(&mut self.f, 0, self.a, machine_cycle),              //BIT 0, A
            0x48 => Cpu::bit_u3_r8(&mut self.f, 1, self.b, machine_cycle),              //BIT 1, B
            0x49 => Cpu::bit_u3_r8(&mut self.f, 1, self.c, machine_cycle),              //BIT 1, C
            0x4A => Cpu::bit_u3_r8(&mut self.f, 1, self.d, machine_cycle),              //BIT 1, D
            0x4B => Cpu::bit_u3_r8(&mut self.f, 1, self.e, machine_cycle),              //BIT 1, E
            0x4C => Cpu::bit_u3_r8(&mut self.f, 1, self.h, machine_cycle),              //BIT 1, H
            0x4D => Cpu::bit_u3_r8(&mut self.f, 1, self.l, machine_cycle),              //BIT 1, L
            0x4E => Cpu::bit_u3_hl(&mut self.f, memory, self.h, self.l, 1, machine_cycle),   //BIT 1, (HL)
            0x4F => Cpu::bit_u3_r8(&mut self.f, 1, self.a, machine_cycle),              //BIT 1, A
            0x50 => Cpu::bit_u3_r8(&mut self.f, 2, self.b, machine_cycle),              //BIT 2, B
            0x51 => Cpu::bit_u3_r8(&mut self.f, 2, self.c, machine_cycle),              //BIT 2, C
            0x52 => Cpu::bit_u3_r8(&mut self.f, 2, self.d, machine_cycle),              //BIT 2, D
            0x53 => Cpu::bit_u3_r8(&mut self.f, 2, self.e, machine_cycle),              //BIT 2, E
            0x54 => Cpu::bit_u3_r8(&mut self.f, 2, self.h, machine_cycle),              //BIT 2, H
            0x55 => Cpu::bit_u3_r8(&mut self.f, 2, self.l, machine_cycle),              //BIT 2, L
            0x56 => Cpu::bit_u3_hl(&mut self.f, memory, self.h, self.l, 2, machine_cycle),   //BIT 2, (HL)
            0x57 => Cpu::bit_u3_r8(&mut self.f, 2, self.a, machine_cycle),              //BIT 2, A
            0x58 => Cpu::bit_u3_r8(&mut self.f, 3, self.b, machine_cycle),              //BIT 3, B
            0x59 => Cpu::bit_u3_r8(&mut self.f, 3, self.c, machine_cycle),              //BIT 3, C
            0x5A => Cpu::bit_u3_r8(&mut self.f, 3, self.d, machine_cycle),              //BIT 3, D
            0x5B => Cpu::bit_u3_r8(&mut self.f, 3, self.e, machine_cycle),              //BIT 3, E
            0x5C => Cpu::bit_u3_r8(&mut self.f, 3, self.h, machine_cycle),              //BIT 3, H
            0x5D => Cpu::bit_u3_r8(&mut self.f, 3, self.l, machine_cycle),              //BIT 3, L
            0x5E => Cpu::bit_u3_hl(&mut self.f, memory, self.h, self.l, 3, machine_cycle),   //BIT 3, (HL)
            0x5F => Cpu::bit_u3_r8(&mut self.f, 3, self.a, machine_cycle),              //BIT 3, A
            0x60 => Cpu::bit_u3_r8(&mut self.f, 4, self.b, machine_cycle),              //BIT 4, B
            0x61 => Cpu::bit_u3_r8(&mut self.f, 4, self.c, machine_cycle),              //BIT 4, C
            0x62 => Cpu::bit_u3_r8(&mut self.f, 4, self.d, machine_cycle),              //BIT 4, D
            0x63 => Cpu::bit_u3_r8(&mut self.f, 4, self.e, machine_cycle),              //BIT 4, E
            0x64 => Cpu::bit_u3_r8(&mut self.f, 4, self.h, machine_cycle),              //BIT 4, H
            0x65 => Cpu::bit_u3_r8(&mut self.f, 4, self.l, machine_cycle),              //BIT 4, L
            0x66 => Cpu::bit_u3_hl(&mut self.f, memory, self.h, self.l, 4, machine_cycle),   //BIT 4, (HL)
            0x67 => Cpu::bit_u3_r8(&mut self.f, 4, self.a, machine_cycle),              //BIT 4, A
            0x68 => Cpu::bit_u3_r8(&mut self.f, 5, self.b, machine_cycle),              //BIT 5, B
            0x69 => Cpu::bit_u3_r8(&mut self.f, 5, self.c, machine_cycle),              //BIT 5, C
            0x6A => Cpu::bit_u3_r8(&mut self.f, 5, self.d, machine_cycle),              //BIT 5, D
            0x6B => Cpu::bit_u3_r8(&mut self.f, 5, self.e, machine_cycle),              //BIT 5, E
            0x6C => Cpu::bit_u3_r8(&mut self.f, 5, self.h, machine_cycle),              //BIT 5, H
            0x6D => Cpu::bit_u3_r8(&mut self.f, 5, self.l, machine_cycle),              //BIT 5, L
            0x6E => Cpu::bit_u3_hl(&mut self.f, memory, self.h, self.l, 5, machine_cycle),   //BIT 5, (HL)
            0x6F => Cpu::bit_u3_r8(&mut self.f, 5, self.a, machine_cycle),              //BIT 5, A
            0x70 => Cpu::bit_u3_r8(&mut self.f, 6, self.b, machine_cycle),              //BIT 6, B
            0x71 => Cpu::bit_u3_r8(&mut self.f, 6, self.c, machine_cycle),              //BIT 6, C
            0x72 => Cpu::bit_u3_r8(&mut self.f, 6, self.d, machine_cycle),              //BIT 6, D
            0x73 => Cpu::bit_u3_r8(&mut self.f, 6, self.e, machine_cycle),              //BIT 6, E
            0x74 => Cpu::bit_u3_r8(&mut self.f, 6, self.h, machine_cycle),              //BIT 6, H
            0x75 => Cpu::bit_u3_r8(&mut self.f, 6, self.l, machine_cycle),              //BIT 6, L
            0x76 => Cpu::bit_u3_hl(&mut self.f, memory, self.h, self.l, 6, machine_cycle),   //BIT 6, (HL)
            0x77 => Cpu::bit_u3_r8(&mut self.f, 6, self.a, machine_cycle),              //BIT 6, A
            0x78 => Cpu::bit_u3_r8(&mut self.f, 7, self.b, machine_cycle),              //BIT 7, B
            0x79 => Cpu::bit_u3_r8(&mut self.f, 7, self.c, machine_cycle),              //BIT 7, C
            0x7A => Cpu::bit_u3_r8(&mut self.f, 7, self.d, machine_cycle),              //BIT 7, D
            0x7B => Cpu::bit_u3_r8(&mut self.f, 7, self.e, machine_cycle),              //BIT 7, E
            0x7C => Cpu::bit_u3_r8(&mut self.f, 7, self.h, machine_cycle),              //BIT 7, H
            0x7D => Cpu::bit_u3_r8(&mut self.f, 7, self.l, machine_cycle),              //BIT 7, L
            0x7E => Cpu::bit_u3_hl(&mut self.f, memory, self.h, self.l, 7, machine_cycle),   //BIT 7, (HL)
            0x7F => Cpu::bit_u3_r8(&mut self.f, 7, self.a, machine_cycle),              //BIT 7, A
            0x80 => Cpu::res_u3_r8(&mut self.b, 0, machine_cycle),                      //RES 0, B
            0x81 => Cpu::res_u3_r8(&mut self.c, 0, machine_cycle),                      //RES 0, C
            0x82 => Cpu::res_u3_r8(&mut self.d, 0, machine_cycle),                      //RES 0, D
            0x83 => Cpu::res_u3_r8(&mut self.e, 0, machine_cycle),                      //RES 0, E
            0x84 => Cpu::res_u3_r8(&mut self.h, 0, machine_cycle),                      //RES 0, H
            0x85 => Cpu::res_u3_r8(&mut self.l, 0, machine_cycle),                      //RES 0, L
            0x86 => Cpu::res_u3_hl(memory, self.h, self.l, 0, machine_cycle),           //RES 0, (HL)
            0x87 => Cpu::res_u3_r8(&mut self.a, 0, machine_cycle),                      //RES 0, A
            0x88 => Cpu::res_u3_r8(&mut self.b, 1, machine_cycle),                      //RES 1, B
            0x89 => Cpu::res_u3_r8(&mut self.c, 1, machine_cycle),                      //RES 1, C
            0x8A => Cpu::res_u3_r8(&mut self.d, 1, machine_cycle),                      //RES 1, D
            0x8B => Cpu::res_u3_r8(&mut self.e, 1, machine_cycle),                      //RES 1, E
            0x8C => Cpu::res_u3_r8(&mut self.h, 1, machine_cycle),                      //RES 1, H
            0x8D => Cpu::res_u3_r8(&mut self.l, 1, machine_cycle),                      //RES 1, L
            0x8E => Cpu::res_u3_hl(memory, self.h, self.l, 1, machine_cycle),           //RES 1, (HL)
            0x8F => Cpu::res_u3_r8(&mut self.a, 1, machine_cycle),                      //RES 1, A
            0x90 => Cpu::res_u3_r8(&mut self.b, 2, machine_cycle),                      //RES 2, B
            0x91 => Cpu::res_u3_r8(&mut self.c, 2, machine_cycle),                      //RES 2, C
            0x92 => Cpu::res_u3_r8(&mut self.d, 2, machine_cycle),                      //RES 2, D
            0x93 => Cpu::res_u3_r8(&mut self.e, 2, machine_cycle),                      //RES 2, E
            0x94 => Cpu::res_u3_r8(&mut self.h, 2, machine_cycle),                      //RES 2, H
            0x95 => Cpu::res_u3_r8(&mut self.l, 2, machine_cycle),                      //RES 2, L
            0x96 => Cpu::res_u3_hl(memory, self.h, self.l, 2, machine_cycle),           //RES 2, (HL)
            0x97 => Cpu::res_u3_r8(&mut self.a, 2, machine_cycle),                      //RES 2, A
            0x98 => Cpu::res_u3_r8(&mut self.b, 3, machine_cycle),                      //RES 3, B
            0x99 => Cpu::res_u3_r8(&mut self.c, 3, machine_cycle),                      //RES 3, C
            0x9A => Cpu::res_u3_r8(&mut self.d, 3, machine_cycle),                      //RES 3, D
            0x9B => Cpu::res_u3_r8(&mut self.e, 3, machine_cycle),                      //RES 3, E
            0x9C => Cpu::res_u3_r8(&mut self.h, 3, machine_cycle),                      //RES 3, H
            0x9D => Cpu::res_u3_r8(&mut self.l, 3, machine_cycle),                      //RES 3, L
            0x9E => Cpu::res_u3_hl(memory, self.h, self.l, 3, machine_cycle),           //RES 3, (HL)
            0x9F => Cpu::res_u3_r8(&mut self.a, 3, machine_cycle),                      //RES 3, A
            0xA0 => Cpu::res_u3_r8(&mut self.b, 4, machine_cycle),                      //RES 4, B
            0xA1 => Cpu::res_u3_r8(&mut self.c, 4, machine_cycle),                      //RES 4, C
            0xA2 => Cpu::res_u3_r8(&mut self.d, 4, machine_cycle),                      //RES 4, D
            0xA3 => Cpu::res_u3_r8(&mut self.e, 4, machine_cycle),                      //RES 4, E
            0xA4 => Cpu::res_u3_r8(&mut self.h, 4, machine_cycle),                      //RES 4, H
            0xA5 => Cpu::res_u3_r8(&mut self.l, 4, machine_cycle),                      //RES 4, L
            0xA6 => Cpu::res_u3_hl(memory, self.h, self.l, 4, machine_cycle),           //RES 4, (HL)
            0xA7 => Cpu::res_u3_r8(&mut self.a, 4, machine_cycle),                      //RES 4, A
            0xA8 => Cpu::res_u3_r8(&mut self.b, 5, machine_cycle),                      //RES 5, B
            0xA9 => Cpu::res_u3_r8(&mut self.c, 5, machine_cycle),                      //RES 5, C
            0xAA => Cpu::res_u3_r8(&mut self.d, 5, machine_cycle),                      //RES 5, D
            0xAB => Cpu::res_u3_r8(&mut self.e, 5, machine_cycle),                      //RES 5, E
            0xAC => Cpu::res_u3_r8(&mut self.h, 5, machine_cycle),                      //RES 5, H
            0xAD => Cpu::res_u3_r8(&mut self.l, 5, machine_cycle),                      //RES 5, L
            0xAE => Cpu::res_u3_hl(memory, self.h, self.l, 5, machine_cycle),           //RES 5, (HL)
            0xAF => Cpu::res_u3_r8(&mut self.a, 5, machine_cycle),                      //RES 5, A
            0xB0 => Cpu::res_u3_r8(&mut self.b, 6, machine_cycle),                      //RES 6, B
            0xB1 => Cpu::res_u3_r8(&mut self.c, 6, machine_cycle),                      //RES 6, C
            0xB2 => Cpu::res_u3_r8(&mut self.d, 6, machine_cycle),                      //RES 6, D
            0xB3 => Cpu::res_u3_r8(&mut self.e, 6, machine_cycle),                      //RES 6, E
            0xB4 => Cpu::res_u3_r8(&mut self.h, 6, machine_cycle),                      //RES 6, H
            0xB5 => Cpu::res_u3_r8(&mut self.l, 6, machine_cycle),                      //RES 6, L
            0xB6 => Cpu::res_u3_hl(memory, self.h, self.l, 6, machine_cycle),           //RES 6, (HL)
            0xB7 => Cpu::res_u3_r8(&mut self.a, 6, machine_cycle),                      //RES 6, A
            0xB8 => Cpu::res_u3_r8(&mut self.b, 7, machine_cycle),                      //RES 7, B
            0xB9 => Cpu::res_u3_r8(&mut self.c, 7, machine_cycle),                      //RES 7, C
            0xBA => Cpu::res_u3_r8(&mut self.d, 7, machine_cycle),                      //RES 7, D
            0xBB => Cpu::res_u3_r8(&mut self.e, 7, machine_cycle),                      //RES 7, E
            0xBC => Cpu::res_u3_r8(&mut self.h, 7, machine_cycle),                      //RES 7, H
            0xBD => Cpu::res_u3_r8(&mut self.l, 7, machine_cycle),                      //RES 7, L
            0xBE => Cpu::res_u3_hl(memory, self.h, self.l, 7, machine_cycle),           //RES 7, (HL)
            0xBF => Cpu::res_u3_r8(&mut self.a, 7, machine_cycle),                      //RES 7, A
            0xC0 => Cpu::set_u3_r8(&mut self.b, 0, machine_cycle),                      //SET 0, B
            0xC1 => Cpu::set_u3_r8(&mut self.c, 0, machine_cycle),                      //SET 0, C
            0xC2 => Cpu::set_u3_r8(&mut self.d, 0, machine_cycle),                      //SET 0, D
            0xC3 => Cpu::set_u3_r8(&mut self.e, 0, machine_cycle),                      //SET 0, E
            0xC4 => Cpu::set_u3_r8(&mut self.h, 0, machine_cycle),                      //SET 0, H
            0xC5 => Cpu::set_u3_r8(&mut self.l, 0, machine_cycle),                      //SET 0, L
            0xC6 => Cpu::set_u3_hl(memory, self.h, self.l, 0, machine_cycle),           //SET 0, (HL)
            0xC7 => Cpu::set_u3_r8(&mut self.a, 0, machine_cycle),                      //SET 0, A
            0xC8 => Cpu::set_u3_r8(&mut self.b, 1, machine_cycle),                      //SET 1, B
            0xC9 => Cpu::set_u3_r8(&mut self.c, 1, machine_cycle),                      //SET 1, C
            0xCA => Cpu::set_u3_r8(&mut self.d, 1, machine_cycle),                      //SET 1, D
            0xCB => Cpu::set_u3_r8(&mut self.e, 1, machine_cycle),                      //SET 1, E
            0xCC => Cpu::set_u3_r8(&mut self.h, 1, machine_cycle),                      //SET 1, H
            0xCD => Cpu::set_u3_r8(&mut self.l, 1, machine_cycle),                      //SET 1, L
            0xCE => Cpu::set_u3_hl(memory, self.h, self.l, 1, machine_cycle),           //SET 1, (HL)
            0xCF => Cpu::set_u3_r8(&mut self.a, 1, machine_cycle),                      //SET 1, A
            0xD0 => Cpu::set_u3_r8(&mut self.b, 2, machine_cycle),                      //SET 2, B
            0xD1 => Cpu::set_u3_r8(&mut self.c, 2, machine_cycle),                      //SET 2, C
            0xD2 => Cpu::set_u3_r8(&mut self.d, 2, machine_cycle),                      //SET 2, D
            0xD3 => Cpu::set_u3_r8(&mut self.e, 2, machine_cycle),                      //SET 2, E
            0xD4 => Cpu::set_u3_r8(&mut self.h, 2, machine_cycle),                      //SET 2, H
            0xD5 => Cpu::set_u3_r8(&mut self.l, 2, machine_cycle),                      //SET 2, L
            0xD6 => Cpu::set_u3_hl(memory, self.h, self.l, 2, machine_cycle),           //SET 2, (HL)
            0xD7 => Cpu::set_u3_r8(&mut self.a, 2, machine_cycle),                      //SET 2, A
            0xD8 => Cpu::set_u3_r8(&mut self.b, 3, machine_cycle),                      //SET 3, B
            0xD9 => Cpu::set_u3_r8(&mut self.c, 3, machine_cycle),                      //SET 3, C
            0xDA => Cpu::set_u3_r8(&mut self.d, 3, machine_cycle),                      //SET 3, D
            0xDB => Cpu::set_u3_r8(&mut self.e, 3, machine_cycle),                      //SET 3, E
            0xDC => Cpu::set_u3_r8(&mut self.h, 3, machine_cycle),                      //SET 3, H
            0xDD => Cpu::set_u3_r8(&mut self.l, 3, machine_cycle),                      //SET 3, L
            0xDE => Cpu::set_u3_hl(memory, self.h, self.l, 3, machine_cycle),           //SET 3, (HL)
            0xDF => Cpu::set_u3_r8(&mut self.a, 3, machine_cycle),                      //SET 3, A
            0xE0 => Cpu::set_u3_r8(&mut self.b, 4, machine_cycle),                      //SET 4, B
            0xE1 => Cpu::set_u3_r8(&mut self.c, 4, machine_cycle),                      //SET 4, C
            0xE2 => Cpu::set_u3_r8(&mut self.d, 4, machine_cycle),                      //SET 4, D
            0xE3 => Cpu::set_u3_r8(&mut self.e, 4, machine_cycle),                      //SET 4, E
            0xE4 => Cpu::set_u3_r8(&mut self.h, 4, machine_cycle),                      //SET 4, H,
            0xE5 => Cpu::set_u3_r8(&mut self.l, 4, machine_cycle),                      //SET 4, L
            0xE6 => Cpu::set_u3_hl(memory, self.h, self.l, 4, machine_cycle),           //SET 4, (HL)
            0xE7 => Cpu::set_u3_r8(&mut self.a, 4, machine_cycle),                      //SET 4, A
            0xE8 => Cpu::set_u3_r8(&mut self.b, 5, machine_cycle),                      //SET 5, B
            0xE9 => Cpu::set_u3_r8(&mut self.c, 5, machine_cycle),                      //SET 5, C
            0xEA => Cpu::set_u3_r8(&mut self.d, 5, machine_cycle),                      //SET 5, D
            0xEB => Cpu::set_u3_r8(&mut self.e, 5, machine_cycle),                      //SET 5, E
            0xEC => Cpu::set_u3_r8(&mut self.h, 5, machine_cycle),                      //SET 5, H
            0xED => Cpu::set_u3_r8(&mut self.l, 5, machine_cycle),                      //SET 5, L
            0xEE => Cpu::set_u3_hl(memory, self.h, self.l, 5, machine_cycle),           //SET 5, (HL)
            0xEF => Cpu::set_u3_r8(&mut self.a, 5, machine_cycle),                      //SET 5, A
            0xF0 => Cpu::set_u3_r8(&mut self.b, 6, machine_cycle),                      //SET 6, B
            0xF1 => Cpu::set_u3_r8(&mut self.c, 6, machine_cycle),                      //SET 6, C
            0xF2 => Cpu::set_u3_r8(&mut self.d, 6, machine_cycle),                      //SET 6, D
            0xF3 => Cpu::set_u3_r8(&mut self.e, 6, machine_cycle),                      //SET 6, E
            0xF4 => Cpu::set_u3_r8(&mut self.h, 6, machine_cycle),                      //SET 6, H
            0xF5 => Cpu::set_u3_r8(&mut self.l, 6, machine_cycle),                      //SET 6, L
            0xF6 => Cpu::set_u3_hl(memory, self.h, self.l, 6, machine_cycle),           //SET 6, (HL)
            0xF7 => Cpu::set_u3_r8(&mut self.a, 6, machine_cycle),                      //SET 6, A
            0xF8 => Cpu::set_u3_r8(&mut self.b, 7, machine_cycle),                      //SET 7, B
            0xF9 => Cpu::set_u3_r8(&mut self.c, 7, machine_cycle),                      //SET 7, C
            0xFA => Cpu::set_u3_r8(&mut self.d, 7, machine_cycle),                      //SET 7, D
            0xFB => Cpu::set_u3_r8(&mut self.e, 7, machine_cycle),                      //SET 7, E
            0xFC => Cpu::set_u3_r8(&mut self.h, 7, machine_cycle),                      //SET 7, H
            0xFD => Cpu::set_u3_r8(&mut self.l, 7, machine_cycle),                      //SET 7, L
            0xFE => Cpu::set_u3_hl(memory, self.h, self.l, 7, machine_cycle),           //SET 7, (HL)
            0xFF => Cpu::set_u3_r8(&mut self.a, 7, machine_cycle),                      //SET 7, A
        }
    }

    /**
     * Rotate register r8 left.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn rlc_r8(flag_reg: &mut u8, reg: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let result = (*reg).rotate_left(1);
                *reg = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(*reg & 0x01 == 0x01));
            },
            _ => panic!("1 to many machine cycles in rlc_r8"),
        }
        return Status::Completed;
    }

    /**
     * Rotate the byte pointed to by HL left.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 2
     */
    fn rlc_hl(flag_reg: &mut u8, memory: &mut Memory, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (), //Should be reading from hl here but nah
            2 => {
                let value = memory.read_byte(build_16bit_num(reg_h, reg_l));
                let result = value.rotate_left(1);
                memory.write_byte(build_16bit_num(reg_h, reg_l), result);
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(result & 0x01 == 0x01));
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in rlc_hl"),
        }
        return Status::Running;
    }

    /**
    * Rotate register r8 right.
    * 
    * MACHINE CYCLE: 2
    * INSTRUCTION LENGTH: 2
    */
    fn rrc_r8(flag_reg: &mut u8, reg: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let result = (*reg).rotate_right(1);
                *reg = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(*reg & 0x80 == 0x80));
            },
            _ => panic!("1 to many machine cycles in rrc_r8"),
        }
        return Status::Completed;
    }


    /**
     * Rotate the byte pointed to by HL right.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 2
     */
    fn rrc_hl(flag_reg: &mut u8, memory: &mut Memory, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (), //Should be reading from hl here but nah
            2 => {
                let hl_data = memory.read_byte(build_16bit_num(reg_h, reg_l));
                let result = hl_data.rotate_right(1);
                memory.write_byte(build_16bit_num(reg_h, reg_l), result);
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(result & 0x80 == 0x80));
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in rrc_hl"),
        }
        return Status::Running;
    }

    /**
     * Rotate register r8 left through carry.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2 
     */
    fn rl_r8(flag_reg: &mut u8, reg: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let rotated_bit: u8 = binary_utils::get_bit(*reg, 7);
                *reg = (*reg << 1) | Cpu::get_carry_flag(*flag_reg);
                Cpu::set_flags(flag_reg, Some(*reg == 0), Some(false), Some(false), Some(rotated_bit != 0));
            },
            _ => panic!("1 to many machine cycles in rl_r8"),
        }
        return Status::Completed;
    }

    /**
     * Rotate the byte pointed to by HL left through carry.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 2
     */
    fn rl_hl(flag_reg: &mut u8, memory: &mut Memory, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (), //Should be reading from hl here but nah
            2 => {
                let hl_data = memory.read_byte(build_16bit_num(reg_h, reg_l));
                let rotated_bit: u8 = binary_utils::get_bit(hl_data, 7);
                let result = (hl_data << 1) | Cpu::get_carry_flag(*flag_reg);
                memory.write_byte(build_16bit_num(reg_h, reg_l), result);

                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(rotated_bit != 0));
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in rl_hl"),
        }
        return Status::Running;
    }

    /**
     * Rotate register r8 right through carry.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn rr_r8(flag_reg: &mut u8, reg: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let rotated_bit: u8 = binary_utils::get_bit(*reg, 0);
                *reg = (*reg >> 1) | (Cpu::get_carry_flag(*flag_reg) << 7);
                Cpu::set_flags(flag_reg, Some(*reg == 0), Some(false), Some(false), Some(rotated_bit != 0));
            },
            _ => panic!("1 to many machine cycles in rr_r8"),
        }
        return Status::Completed;
    }

    
    /**
     * Rotate the byte pointed to by HL right through carry.
     * 
     * MACHINE CYCLE: 4
     * INSTRUCTION LENGTH: 2
     */
    fn rr_hl(flag_reg: &mut u8, memory: &mut Memory, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (), //Should be reading from hl here but nah
            2 => {
                let hl_data = memory.read_byte(build_16bit_num(reg_h, reg_l));
                let rotated_bit: u8 = binary_utils::get_bit(hl_data, 0);
                let result = (hl_data >> 1) | (Cpu::get_carry_flag(*flag_reg) << 7);
                memory.write_byte(build_16bit_num(reg_h, reg_l), result);

                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(rotated_bit != 0));
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in rr_hl"),
        }
        return Status::Running;
    }

    /**
     * Shift Left Arithmetically register r8.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn sla_r8(flag_reg: &mut u8, reg: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let rotated_bit: u8 = binary_utils::get_bit(*reg, 7);
                *reg = *reg << 1;
                Cpu::set_flags(flag_reg, Some(*reg == 0), Some(false), Some(false), Some(rotated_bit != 0));
            },
            _ => panic!("1 to many machine cycles in sla_r8"),
        }
        return Status::Completed;
    }

    /**
     * Shift Left Arithmetically the byte pointed to by HL.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 2
     */
    fn sla_hl(flag_reg: &mut u8, memory: &mut Memory, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (), //Should be reading from hl here but nah
            2 => {
                let hl_data = memory.read_byte(build_16bit_num(reg_h, reg_l));
                let rotated_bit: u8 = binary_utils::get_bit(hl_data, 7);
                let result = hl_data << 1;
                memory.write_byte(build_16bit_num(reg_h, reg_l), result);

                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(rotated_bit != 0));
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in sla_hl"),
        }
        return Status::Running;
    }

    /**
     * Shift Right Arithmetically register r8.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn sra_r8(flag_reg: &mut u8, reg: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let rotated_bit: u8 = binary_utils::get_bit(*reg, 0);
                *reg = (*reg >> 1) | (*reg & 0x80);
                Cpu::set_flags(flag_reg, Some(*reg == 0), Some(false), Some(false), Some(rotated_bit != 0));
            },
            _ => panic!("1 to many machine cycles in sra_r8"),
        }
        return Status::Completed;
    }

    /**
     * Shift Right Arithmetically the byte pointed to by HL.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 2
     */
    fn sra_hl(flag_reg: &mut u8, memory: &mut Memory, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (), //Should be reading from hl here but nah
            2 => {
                let hl_data = memory.read_byte(build_16bit_num(reg_h, reg_l));
                let rotated_bit: u8 = binary_utils::get_bit(hl_data, 0);
                let result = (hl_data >> 1) | (hl_data & 0x80);
                memory.write_byte(build_16bit_num(reg_h, reg_l), result);

                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(rotated_bit != 0));
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in sra_hl"),
        }
        return Status::Running;
    }

    /**
     * Swap the upper 4 bits in register r8 and the lower 4 ones.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn swap_r8(flag_reg: &mut u8, reg: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let lower_nibble = *reg >> 4;
                let result = (*reg << 4) | lower_nibble;
                *reg = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(false));
            },
            _ => panic!("1 to many machine cycles in swap_r8"),
        }
        return Status::Completed;
    }

    /**
     * Swap the upper 4 bits in the byte pointed to by HL and the lower 4 ones.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 2
     */
    fn swap_hl(flag_reg: &mut u8, memory: &mut Memory, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (), //Should be reading from hl here but nah
            2 => {
                let hl_data = memory.read_byte(build_16bit_num(reg_h, reg_l));
                let lower_nibble = hl_data >> 4;
                let result = (hl_data << 4) | lower_nibble;
                memory.write_byte(build_16bit_num(reg_h, reg_l), result);

                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(false));
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in swap_hl"),
        }
        return Status::Running;
    }
   

    /**
     * Shift Right Logically register r8.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn srl_r8(flag_reg: &mut u8, reg: &mut u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let rotated_bit: u8 = binary_utils::get_bit(*reg, 0);
                *reg = *reg >> 1;
                Cpu::set_flags(flag_reg, Some(*reg == 0), Some(false), Some(false), Some(rotated_bit != 0));
            },
            _ => panic!("1 to many machine cycles in srl_r8"),
        }
        return Status::Completed;
    }

    /**
     * Shift Right Logically the byte pointed to by HL.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 2
     */
    fn srl_hl(flag_reg: &mut u8, memory: &mut Memory, reg_h: u8, reg_l: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (), //Should be reading from hl here but nah
            2 => {
                let hl_data = memory.read_byte(build_16bit_num(reg_h, reg_l));
                let rotated_bit: u8 = binary_utils::get_bit(hl_data, 0);
                let result = hl_data >> 1;
                memory.write_byte(build_16bit_num(reg_h, reg_l), result);

                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(rotated_bit != 0));
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in srl_hl"),
        }
        return Status::Running;
    }

    /**
     * Bit test register r8.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn bit_u3_r8(flag_reg: &mut u8, bit_to_test: u8, reg: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let result = binary_utils::get_bit(reg, bit_to_test);
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(true), None);
            },
            _ => panic!("1 to many machine cycles in bit_u3_r8"),
        }
        return Status::Completed;
    }

    /**
     * Bit test the byte pointed to by HL.
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 2
     */
    fn bit_u3_hl(flag_reg: &mut u8, memory: &Memory, reg_h: u8, reg_l: u8, bit_to_test: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let hl_data = memory.read_byte(build_16bit_num(reg_h, reg_l));
                let result = binary_utils::get_bit(hl_data, bit_to_test);
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(true), None);
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in bit_u3_hl"),
        }
    }

    /**
     * Reset bit u3 in register r8 to 0. Bit 0 is the rightmost one, bit 7 the leftmost one.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn res_u3_r8(reg: &mut u8, bit_to_reset: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let result = *reg & !(0x1 << bit_to_reset);
                *reg = result;
            }, 
            _ => panic!("1 to many machine cycles in res_u3_r8"),
        }
        return Status::Completed;
    }

    /**
     * Reset bit u3 in the byte pointed to by HL to 0. Bit 0 is the rightmost one, bit 7 the leftmost one.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 2
     */
    fn res_u3_hl(memory: &mut Memory, reg_h: u8, reg_l: u8, bit_to_reset: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (), //Should be reading from hl here but nah
            2 => {
                let hl_data = memory.read_byte(build_16bit_num(reg_h, reg_l));
                let result = hl_data & !(0x1 << bit_to_reset);
                memory.write_byte(build_16bit_num(reg_h, reg_l), result);
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in res_u3_hl"),
        }
        return Status::Running;
    }

    /**
     * Set bit u3 in register r8 to 1. Bit 0 is the rightmost one, bit 7 the leftmost one.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn set_u3_r8(reg: &mut u8, bit_to_set: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => {
                let result = *reg | (0x1 << bit_to_set);
                *reg = result;
            },
            _ => panic!("1 to many machine cycles in set_u3_r8"),
        }
        return Status::Completed;
    }

    /**
     * Set bit u3 in the byte pointed to by HL to 1. Bit 0 is the rightmost one, bit 7 the leftmost one.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 2
     */
    fn set_u3_hl(memory: &mut Memory, reg_h: u8, reg_l: u8, bit_to_set: u8, machine_cycle: u8) -> Status {
        match machine_cycle {
            1 => (), //Should be reading from hl here but nah
            2 => {
                let hl_data = memory.read_byte(build_16bit_num(reg_h, reg_l));
                let result = hl_data | (0x1 << bit_to_set);
                memory.write_byte(build_16bit_num(reg_h, reg_l), result);
                return Status::Completed;
            },
            _ => panic!("1 to many machine cycles in set_u3_hl"),
        }
        return Status::Running;
    }















    fn get_zero_flag(flag_reg: u8) -> u8 {
        (flag_reg >> 7) & 0x1
    }

    fn get_negative_flag(flag_reg: u8) -> u8 {
        (flag_reg >> 6) & 0x1
    }

    fn get_half_carry_flag(flag_reg: u8) -> u8 {
        (flag_reg >> 5) & 0x1
    }

    fn get_carry_flag(flag_reg: u8) -> u8 {
        (flag_reg >> 4) & 0x1
    }
}

