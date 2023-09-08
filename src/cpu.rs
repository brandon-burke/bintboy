use core::panic;
use std::f32::consts::E;

use crate::memory::Memory;
use crate::cpu_state::{CpuState, ExecuteStatus};
use crate::opcodes::{OPCODE_MACHINE_CYCLES, PREFIX_OPCODE_MACHINE_CYCLES};
use crate::binary_utils::{self, split_16bit_num, build_16bit_num};

const MACHINE_CYCLE: u8 = 4;
const PREFIX_OPCODE: u8 = 0xCB;

pub struct Cpu {
    pub a: u8,              //Accumulator Register
    pub b: u8,              //General Purpose Register
    pub c: u8,              //General Purpose Register
    pub d: u8,              //General Purpose Register
    pub e: u8,              //General Purpose Register
    pub f: u8,              //Flags Register
    pub h: u8,              //General Purpose Register
    pub l: u8,              //General Purpose Register
    pub sp: u16,            //Stack Pointer Register
    pub pc: u16,            //Program Counter Register
    cpu_state: CpuState,    //Let's us know the current state of the CPU
    cpu_clk_cycles: u8,     //Keeps track of how many cpu clk cycles have gone by
    current_opcode: u8,     //Keeps track of the current worked on opcode
    ime_flag: bool,              //Interrupt Master Enable
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
            ime_flag: false,
        }
    }

    pub fn cycle(&mut self, memory: &mut Memory) {

        /* Have to wait 1 machine cycle before we do anywork */
        self.cpu_clk_cycles += 1;
        if self.cpu_clk_cycles >= MACHINE_CYCLE {
            self.cpu_clk_cycles = 0;
        } else {
            return;
        }

        //Depending on what state you are in you have to do the work that corresponds to it
        match self.cpu_state {
            CpuState::Fetch => {
                self.current_opcode = self.fetch(memory);
                
                if self.current_opcode == PREFIX_OPCODE {
                    self.cpu_state = CpuState::FetchPrefix;
                } else {
                    self.cpu_state = CpuState::Execute { machine_cycle: 0, temp_reg: 0 };

                    if OPCODE_MACHINE_CYCLES[self.current_opcode as usize] == 1 {
                        match self.exexute(memory, 1, &mut 0) {
                            ExecuteStatus::Completed => self.cpu_state = CpuState::InterruptHandle,
                            ExecuteStatus::Running => (),
                            ExecuteStatus::Error => panic!("Error Executing opcode"),
                        }
                    }
                }
            },
            CpuState::FetchPrefix => {
                self.current_opcode = self.fetch(memory);
                self.cpu_state = CpuState::Execute { machine_cycle: 0, temp_reg: 0 };

                if PREFIX_OPCODE_MACHINE_CYCLES[self.current_opcode as usize] == 2 { //2 b/c you always have to fetch the prefix
                    match self.exexute_prefix(memory, 1, &mut 0) {
                        ExecuteStatus::Completed => self.cpu_state = CpuState::InterruptHandle,
                        ExecuteStatus::Running => (),
                        ExecuteStatus::Error => panic!("Error Executing opcode"),
                    }
                }
            },
            CpuState::Execute { mut machine_cycle, mut temp_reg } => {
                machine_cycle += 1;
                match self.exexute(memory, machine_cycle, &mut temp_reg) {
                    ExecuteStatus::Completed => self.cpu_state = CpuState::InterruptHandle,
                    ExecuteStatus::Running => (),
                    ExecuteStatus::Error => panic!("Error Executing opcode"),
                }    
            },
            CpuState::InterruptHandle => todo!(),
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
    pub fn exexute(&mut self, memory: &mut Memory, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
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
            0x10 => Cpu::stop(),                                                                        //STOP
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
            0x3E => Cpu::ld_r8_u8(memory, &mut self.a, &mut self.pc, machine_cycle),            //LD_A_U81
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
            0x6B => Cpu::ld_r8_r8(self.e, &mut self.e, machine_cycle),                          //LD_L_E
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
            0x76 => Cpu::halt(),                                                                //HALT
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
            0xCB => Cpu::prefix(memory),    //Need to pass the self.current opcode here as that will
            0xCC => Cpu::call_cc_u16(memory, &mut self.pc, &mut self.sp, Cpu::get_zero_flag(self.f) != 0, machine_cycle, temp_reg),
            0xCD => Cpu::call_u16(memory, &mut self.pc, &mut self.sp, machine_cycle, temp_reg),   //CALL_U16
            0xCE => Cpu::adc_a_u8(&mut self.f, memory, &mut self.a, &mut self.pc, &mut self.sp, machine_cycle),   //ADC_A_U8
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
            0xD9 => Cpu::reti(&mut self.ime_flag, memory, &mut self.sp, &mut self.pc, machine_cycle, temp_reg), //RETI
            0xDA => Cpu::jp_cc_u16(memory, &mut self.pc, Cpu::get_carry_flag(self.f) != 0, machine_cycle, temp_reg),             //JP_C_U16
            0xDB => panic!("0xDB is an unused opcode"),
            0xDC => Cpu::call_cc_u16(memory, &mut self.pc, &mut self.sp, Cpu::get_carry_flag(self.f) != 0, machine_cycle, temp_reg),    //CALL_C_U16
            0xDD => panic!("0xDD is an unused opcode"),
            0xDE => Cpu::sbc_a_u8(&mut self.f, memory, &mut self.a, &mut self.pc, machine_cycle),   //SBC_A_U8
            0xDF => Cpu::rst_vec(memory, &mut self.sp, &mut self.pc, 0x18, machine_cycle),  //RST_18
            0xE0 => Cpu::ldh_u8_a(memory, &mut self.pc, &mut self.a, machine_cycle, temp_reg),         //LDH_U8_A
            0xE1 => Cpu::pop(memory, &mut self.h, &mut self.l, &mut self.sp, machine_cycle),    //POP_HL
            0xE2 => Cpu::ldh_c_a(memory, self.a, self.c, machine_cycle, temp_reg),       //LDH_(0xFF00+C)_A
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
            0xF1 => Cpu::pop(memory, &mut self.a, &mut self.f, &mut self.sp, machine_cycle),    //POP_AF
            0xF2 => Cpu::ldh_a_c(memory, &mut self.a, self.c, machine_cycle, temp_reg),       //LDH_A_(0xFF00+C)
            0xF3 => Cpu::di(&mut self.ime_flag, machine_cycle),                             //DI
            0xF4 => panic!("0xF4 is an unused opcode"),
            0xF5 => Cpu::push_r16(memory, self.a, self.f, &mut self.sp, machine_cycle), //PUSH_AF
            0xF6 => Cpu::or_a_u8(&mut self.f, memory, &mut self.a, &mut self.pc, machine_cycle),    //OR_A_U8
            0xF7 => Cpu::rst_vec(memory, &mut self.sp, &mut self.pc, 0x30, machine_cycle),  //RST_30
            0xF8 => Cpu::ld_hl_sp_i8(&mut self.f, memory, &mut self.sp, &mut self.pc, &mut self.h, &mut self.l, machine_cycle), //LD_HL_SP_I8
            0xF9 => Cpu::ld_sp_hl(self.h, self.l, &mut self.sp, machine_cycle),            //LD_SP_HL
            0xFA => Cpu::ld_a_u16(memory, &mut self.pc, &mut self.a, machine_cycle, temp_reg),      //LD_A_U16
            0xFB => Cpu::ei(&mut self.ime_flag, machine_cycle),                             //EI
            0xFC => panic!("0xFC is an unused opcode"),
            0xFD => panic!("0xFD is an unused opcode"),
            0xFE => Cpu::cp_a_u8(&mut self.f, memory, &mut self.a, &mut self.pc, machine_cycle),   //CP_A_U8
            0xFF => Cpu::rst_vec(memory, &mut self.sp, &mut self.pc, 0x38, machine_cycle),  //RST_38
            _ => panic!("Unknown opcode"),
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
    fn nop(machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => (),
            _ => panic!("1 to many machine cycles on nop"),
        }
        return ExecuteStatus::Completed
    }

    /**
     * Loads the unsigned 16 bit value into the given registers
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 3
     */
    fn ld_r16_u16(memory: &Memory, upper_reg: &mut u8, lower_reg: &mut u8, pc: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => *lower_reg = memory.read_byte(*pc),
            2 => { 
                *upper_reg = memory.read_byte(*pc); 
                return ExecuteStatus::Completed; 
            },
            _ => panic!("1 to many cycles on ld_r16_u16"),
        }
        *pc += 1;
        return ExecuteStatus::Running;
    }

    /**
     * Store the value in register A into the byte pointed to by register r16.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 3
     */
    fn ld_r16_a(memory: &mut Memory, a_reg: u8, upper_reg: u8, lower_reg: u8, machine_cycle: u8) -> ExecuteStatus {
        let address: u16 = (upper_reg as u16) << 8 | lower_reg as u16;
        match machine_cycle {
            1 => memory.write_byte(address, a_reg),
            _ => panic!("1 to many cycles on ld_r16_a") 
        }

        return ExecuteStatus::Completed;
    }

    /**
     * Increment value in register r16 by 1. PROBABLY NOT CYCLE ACCURATE
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn inc_r16(upper_reg: &mut u8, lower_reg: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let r16 = binary_utils::build_16bit_num(*upper_reg, *lower_reg) + 1;
                *upper_reg = (r16 >> 8) as u8;
                *lower_reg = r16 as u8;
            }
            _ => panic!("1 to many cycles on inc_r16"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Increment value in register r8 by 1.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn inc_r8(flag_reg: &mut u8, reg: &mut u8,  machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => *reg += 1,
            _ => panic!("1 to many cycles on inc_r8"),
        }
        Cpu::set_flags(flag_reg, Some(*reg == 0), Some(false), Some((*reg & 0xF) + 1 > 0xF), None);
        return ExecuteStatus::Completed;
    }

    /**
     * Decrement value in register r8 by 1.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    pub fn dec_r8(flag_reg: &mut u8, reg: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => *reg -= 1,           
            _ => panic!("1 to many cycles on dec_r8"),
        }
        Cpu::set_flags(flag_reg, Some(*reg == 0), Some(true), Some(*reg & 0xF == 0xF), None);
        return ExecuteStatus::Completed;
    }

    /**
     * Load value u8 into register r8
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn ld_r8_u8(memory: &Memory, reg: &mut u8, pc: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => *reg = memory.read_byte(*pc),
            _ => panic!("1 to many cycles on ld_r8_u8"),
        }
        *pc += 1;
        return ExecuteStatus::Completed;
    }

    /**
     * Rotate register A left.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn rlca(flag_reg: &mut u8, reg_a: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                Cpu::set_flags(flag_reg, Some(false), Some(false), Some(false), Some(binary_utils::get_bit(*reg_a, 7) != 0));
                *reg_a = (*reg_a).rotate_left(1);
            }
            _ => panic!("1 to many cycles on RLCA"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Store SP & $FF at address n16 and SP >> 8 at address n16 + 1.
     * 
     * MACHINE CYCLES: 5
     * INSTRUCTION LENGTH: 3
     */
    fn ld_u16_sp(memory: &mut Memory, pc: &mut u16, sp: u16, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
        let mut status = ExecuteStatus::Running;
        match machine_cycle {
            1 => { *temp_reg |= memory.read_byte(*pc) as u16; *pc += 1; },        //read lower byte ASSUMING TEMP REG TO BE 0
            2 => { *temp_reg |= (memory.read_byte(*pc) as u16) << 8; *pc += 1; }, //read upper byte 
            3 => memory.write_byte(*temp_reg, sp as u8),
            4 => {
                memory.write_byte(*temp_reg + 1, (sp >> 8) as u8);
                status = ExecuteStatus::Completed;
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
    fn add_hl_r16(flag_reg: &mut u8, upper_reg: u8, lower_reg: u8, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> ExecuteStatus {
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

        return ExecuteStatus::Completed;
    }

    /**
     * Load value in register A from the byte pointed to by register r16.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_a_r16(memory: &Memory, reg_a: &mut u8, upper_reg: u8, lower_reg: u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => *reg_a = memory.read_byte(binary_utils::build_16bit_num(upper_reg, lower_reg)),
            _ => panic!("1 to many cycles on ld_a_r16"),
        }
        return ExecuteStatus::Completed;
    }

    /**
    * Decrement value in register r16 by 1.
    * 
    * MACHINE CYCLES: 2
    * INSTRUCTION LENGTH: 1
    */
    fn dec_r16(upper_reg: &mut u8, lower_reg: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let r16 = binary_utils::build_16bit_num(*upper_reg, *lower_reg) - 1;
                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(r16);
                *upper_reg = upper_byte;
                *lower_reg = lower_byte;
            }
            _ => panic!("1 to many cycles on dec_r16"),
        }  

        return ExecuteStatus::Completed;
    }

    /**
    * Rotate register A right.
    * 
    * MACHINE CYCLE: 1
    * INSTRUCTION LENGTH: 1
    */
    fn rrca(flag_reg: &mut u8, reg_a: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                Cpu::set_flags(flag_reg, Some(false), Some(false), Some(false), Some(binary_utils::get_bit(*reg_a, 0) != 0));
                *reg_a = (*reg_a).rotate_right(1);
            }
            _ => panic!("1 to many cycles on RLCA"),
        }

        return ExecuteStatus::Completed;
    }

    /**
     * THIS IS VERY SPECIAL NEED TO KNOW MORE ABOUT IT. Helps the gameboy
     * get into a very low power state, but also turns off a lot of peripherals
     */
    fn stop() -> ExecuteStatus {
        //Need to reset the Timer divider register
        //timer begins ticking again once stop mode ends
        todo!("NEED TO IMPLEMENT THE STOP INSTRUCTION");
    }

    /**
     * Rotate register A left through carry.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1 
     */
    fn rla(flag_reg: &mut u8, reg_a: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                Cpu::set_flags(flag_reg, Some(false), Some(false), Some(false), Some(binary_utils::get_bit(*reg_a, 7) != 0));
                *reg_a = (*reg_a << 1) | Cpu::get_carry_flag(*flag_reg);
            }
            _ => panic!("1 to many machine cycles in rla")
        }

        return ExecuteStatus::Completed;
    }

    /**
     * Jump by i8 to a different address relative to the pc 
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 2
     */
    fn jr_i8(memory: &Memory, pc: &mut u16, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
        let mut status = ExecuteStatus::Running;
        match machine_cycle {
            1 => { *temp_reg = memory.read_byte(*pc) as u16; *pc += 1; },
            2 => {
                *pc = (*pc).wrapping_add_signed(*temp_reg as i16);
                status = ExecuteStatus::Completed;
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
    fn rra(flag_reg: &mut u8, reg_a: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                Cpu::set_flags(flag_reg, Some(false), Some(false), Some(false), Some(binary_utils::get_bit(*reg_a, 0) != 0));
                *reg_a = (*reg_a >> 1) | (Cpu::get_carry_flag(*flag_reg) << 7);
            }
            _ => panic!("1 to many machine cycles in rla")
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Relative Jump by i8 if condition cc is met.
     * 
     * MACHINE CYCLES: 3 IF TAKEN/ 2 IF NOT TAKEN
     * INSTRUCTION LENGTH: 2
     */
    fn jr_cc_i8(memory: &Memory, pc: &mut u16, condition: bool, machine_cycle: u8, temp_reg: &mut u16)  -> ExecuteStatus {
        let mut status = ExecuteStatus::Running;
        match machine_cycle {
            1 => { 
                *temp_reg = memory.read_byte(*pc) as u16; 
                *pc += 1; 
                if !condition {
                    status = ExecuteStatus::Completed;
                }
            },
            2 => {
                *pc = (*pc).wrapping_add_signed(*temp_reg as i16);
                status = ExecuteStatus::Completed;
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
    fn ld_hli_a(memory: &mut Memory, reg_a: &mut u8, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let reg_hl = binary_utils::build_16bit_num(*reg_h, *reg_l);
                memory.write_byte(reg_hl, *reg_a);

                let (upper_byte, lower_byte) = split_16bit_num(reg_hl + 1);
                *reg_h = upper_byte;
                *reg_l = lower_byte;
            },
            _ => panic!("1 to many machine cycles in ld_hli_a"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Decimal Adjust Accumulator to get a correct BCD representation after an arithmetic instruction.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn daa(flag_reg: &mut u8, reg_a: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let mut carry_flag = false;
                if Cpu::get_negative_flag(*flag_reg) == 0 {
                    if Cpu::get_carry_flag(*flag_reg) != 0 || *reg_a > 0x99 {
                        *reg_a += 0x60;
                        carry_flag = true;
                    }
                    if Cpu::get_half_carry_flag(*flag_reg) != 0 || *reg_a & 0x0F > 0x09 {
                        *reg_a += 0x6; 
                    }
                }
                Cpu::set_flags(flag_reg, Some(*reg_a == 0), None, Some(false), Some(carry_flag));
            },
            _ => panic!("1 to many machine cycles in daa"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Load value into register A from the byte pointed by HL and increment HL afterwards.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_a_hli(memory: &Memory, reg_a: &mut u8, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> ExecuteStatus {
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
        return ExecuteStatus::Completed;                     
    }

    /**
     * Store the complement of the A register into the A register
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn cpl(flag_reg: &mut u8, reg_a: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                *reg_a = !*reg_a;
                Cpu::set_flags(flag_reg, None, Some(true), Some(true), None);
            },
            _ => panic!("1 to many machine cycles in cpl"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Load value n16 into register SP.
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 3
     */
    fn ld_sp_u16(memory: &Memory, pc: &mut u16, sp: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => *sp = memory.read_byte(*pc) as u16,
            2 => *sp |= (memory.read_byte(*pc) as u16) << 8,
            _ => panic!("1 to many machine cycles in ld_sp_u16"),
        }
        *pc += 1;
        return ExecuteStatus::Running;
    }

    /**
     * Store value in register A into the byte pointed by HL and decrement HL afterwards.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_hld_a(memory: &mut Memory, reg_a: &mut u8, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> ExecuteStatus {
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
        return ExecuteStatus::Completed;
    }

    /**
     * Increment value in register SP by 1. NOT ACCURATE
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn inc_sp(sp: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => *sp += 1,
            _ => panic!("1 to many machine cycles in inc_sp"), 
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Increment the byte pointed to by HL by 1.
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 1
     */
    fn inc_hl(flag_reg: &mut u8, memory: &mut Memory, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => (), //should be reading from HL here
            2 => {
                let reg_hl = binary_utils::build_16bit_num(*reg_h, *reg_l);
                let mut hl_data = memory.read_byte(reg_hl);
                hl_data += 1;
                memory.write_byte(reg_hl, hl_data);
                Cpu::set_flags(flag_reg, Some(hl_data == 0), Some(false), Some((hl_data & 0xF) + 1 > 0xF), None);

                return ExecuteStatus::Completed;
            },
            _ => panic!("1 to many machine cycles in inc_hl"),
        }
        return ExecuteStatus::Running;
    }

    /**
    * Decrement the byte pointed to by HL by 1.
    * 
    * MACHINE CYCLES: 3
    * INSTRUCTION LENGTH: 1
    */
    fn dec_hl(flag_reg: &mut u8, memory: &mut Memory, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => (), //read byte at HL. too lazy to implement temp reg at this step just doing it on next step
            2 => {
                let reg_hl = binary_utils::build_16bit_num(*reg_h, *reg_l);
                let hl_data = memory.read_byte(reg_hl) - 1;
                memory.write_byte(reg_hl, hl_data);

                Cpu::set_flags(flag_reg, Some(hl_data == 0), Some(true), Some(hl_data & 0xF == 0xF), None);
                return ExecuteStatus::Completed;
            },
            _ => panic!("1 to many machine cycles in dec_hl"),
        }
        return ExecuteStatus::Running;
    }

    /**
     * Store value u8 into the byte pointed to by register HL.
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 2
     */
    fn ld_hl_u8(memory: &mut Memory, reg_h: u8, reg_l: u8, pc: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => (),    //Should be reading immediate here
            2 => {
                let reg_hl = binary_utils::build_16bit_num(reg_h, reg_l);
                let immediate = memory.read_byte(*pc);
                *pc += 1;
                memory.write_byte(reg_hl, immediate);

                return ExecuteStatus::Completed; 
            },
            _ => panic!("1 to many machine cycles in ld_hl_u8"),
        }
        return ExecuteStatus::Running;
    }

        /**
     * Set the carry flag
     * 
     * MACHINE CYCLE: 1
     * INSTRUCTION LENGTH: 1
     */
    fn scf(flag_reg: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => Cpu::set_flags(flag_reg, None, Some(false), Some(false), Some(true)),
            _ => panic!("1 to many machine cycles in scf"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Add the value in sp to hl. Probably not cycle accurate
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn add_hl_sp(flag_reg: &mut u8, reg_h: &mut u8, reg_l: &mut u8, sp: &mut u16, machine_cycle: u8) -> ExecuteStatus {
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
        return ExecuteStatus::Completed;
    }

    /**
     * Load value into register A from the byte pointed by HL and decrement HL afterwards.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_a_hld(memory: &Memory, reg_a: &mut u8, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> ExecuteStatus {
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
        return ExecuteStatus::Completed;
    }

 /**
    * Decrement value in register SP by 1.
    * 
    * MACHINE CYCLES: 2
    * INSTRUCTION LENGTH: 1
    */
    pub fn dec_sp(sp: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                *sp -= 1;
            }
            _ => panic!("1 to many machine cycles in dec_sp"),
        }
        return ExecuteStatus::Completed;
    }

    /**
    * Complement the carry flag
    */
    fn ccf(flag_reg: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => Cpu::set_flags(flag_reg, None, Some(false), Some(false), Some(Cpu::get_carry_flag(*flag_reg) == 0)),
            _ => panic!("1 to many machine cycles in ccf"), 
        }   
        return ExecuteStatus::Completed;
    }

    /**
     * Load (copy) value in register on the right into register on the left.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn ld_r8_r8(reg_right: u8, reg_left: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => *reg_left = reg_right,
            _ => panic!("1 to many machine cycles in ld_r8_r8"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Load value into register r8 from the byte pointed to by register HL.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_r8_hl(memory: &Memory, reg_h: u8, reg_l: u8, reg: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => *reg = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l)),
            _ => panic!("1 to many machine cycles in ld_r8_hl"),
        }
        return ExecuteStatus::Completed;
    }

        /**
     * Store value in register r8 into the byte pointed to by register HL.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_hl_r8(memory: &mut Memory, reg: u8, reg_h: u8, reg_l: u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => memory.write_byte(binary_utils::build_16bit_num(reg_h, reg_l), reg),
            _ => panic!("1 to many machine cycles in ld_hl_r8"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Enter CPU low-power consumption mode until an interrupt occurs. 
     * The exact behavior of this instruction depends on the state of the IME flag.
     * 
     * MACHINE CYCLES: -
     * INSTRUCTION LENGTH: 1
     */
    fn halt() -> ExecuteStatus {
        todo!();
        return ExecuteStatus::Completed;
    }

    /**
     * Add the value in r8 to A.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1 
     */
    fn add_a_r8(flag_reg: &mut u8, reg: u8, reg_a: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let (result, overflow) = (*reg_a).overflowing_add(reg); 
                let half_carry_overflow = (*reg_a & 0xF) + (reg & 0xF) > 0xF;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(half_carry_overflow), Some(overflow));
            },
            _ => panic!("1 to many machine cycles in add_a_r8"),
        }
        return ExecuteStatus::Completed;
    }

    /**
    * Add the byte pointed to by HL to A.
    * 
    * MACHINE CYCLES: 2
    * INSTRUCTION LENGTH: 1 
    */
    fn add_a_hl(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, reg_h: u8, reg_l: u8, machine_cycle: u8) -> ExecuteStatus {
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
        return ExecuteStatus::Completed;
    }

    /**
     * Add the value in r8 plus the carry flag to A.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1 
     */
    fn adc_a_r8(flag_reg: &mut u8, reg: u8, reg_a: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let (partial_result, first_overflow) = (*reg_a).overflowing_add(reg);
                let (result, second_overflow) = partial_result.overflowing_add(Cpu::get_carry_flag(*flag_reg));
                let half_carry_overflow = (*reg_a & 0xF) + (reg & 0xF) + Cpu::get_carry_flag(*flag_reg) > 0xF;
        
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(half_carry_overflow), Some(second_overflow));
            },
            _ => panic!("1 to many machine cycles in adc_a_r8"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Add the byte pointed to by HL plus the carry flag to A.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1 
     */
    fn adc_a_hl(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, reg_h: u8, reg_l: u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                let (partial_result, first_overflow) = (*reg_a).overflowing_add(hl_data);
                let (result, second_overflow) = partial_result.overflowing_add(Cpu::get_carry_flag(*flag_reg));
                let half_carry_overflow = (*reg_a & 0xF) + (hl_data & 0xF) + Cpu::get_carry_flag(*flag_reg) > 0xF;
        
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(half_carry_overflow), Some(second_overflow));
            },
            _ => panic!("1 to many machine cycles in adc_a_hl"),
        }
        return ExecuteStatus::Completed;
    }

    /**
    * Subtract the value in r8 from A.
    * 
    * MACHINE CYCLES: 1
    * INSTRUCTION LENGTH: 1
    */
    fn sub_a_r8(flag_reg: &mut u8, reg: u8, reg_a: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let result = *reg_a - reg;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some(reg > *reg_a & 0xF), Some(reg > *reg_a));
            },
            _ => panic!("1 to many machine cycles in sub_a_r8"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Subtract the byte pointed to by HL from A.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn sub_a_hl(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, reg_h: u8, reg_l: u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                let result = *reg_a - hl_data;
                *reg_a = result;

                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(hl_data > *reg_a & 0xF), Some(hl_data > *reg_a));
            },
            _ => panic!("1 to many machine cycles in sub_a_hl"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Subtract the value in r8 and the carry flag from A.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn sbc_a_r8(flag_reg: &mut u8, reg: u8, reg_a: &mut u8 ,machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let result = *reg_a - reg - Cpu::get_carry_flag(*flag_reg);
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some((reg - Cpu::get_carry_flag(*flag_reg)) > *reg_a & 0xF), Some(reg + Cpu::get_carry_flag(*flag_reg) > *reg_a));
            },
            _ => panic!("1 to many machine cycles in sbc_a_r8"),
        }
        return ExecuteStatus::Completed;
    } 

    /**
     * Subtract the byte pointed to by HL and the carry flag from A.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn sbc_a_hl(flag_reg: &mut u8, reg_h: u8, reg_l: u8, reg_a: &mut u8, memory: &Memory, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                let result = *reg_a - hl_data - Cpu::get_carry_flag(*flag_reg);
        
                *reg_a = result;

                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some((hl_data - Cpu::get_carry_flag(*flag_reg)) > *reg_a & 0xF), Some(hl_data > *reg_a));
            },
            _ => panic!("1 to many machine cycles in sbc_a_r8"),
        }
        return ExecuteStatus::Completed;
    }

/**
     * Bitwise AND between the value in r8 and A.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn and_a_r8(flag_reg: &mut u8, reg: u8, reg_a: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let result = reg & *reg_a;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(true), Some(false));
            } ,
            _ => panic!("1 to many machine cycles in and_a_r8"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Bitwise AND between the byte pointed to by HL and A.
     * 
     * MACHINE CYCLE: 2
     * INSTRUCTION LENGTH: 1
     */
    fn and_a_hl(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, reg_h: u8, reg_l: u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle { 
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                let result = hl_data & *reg_a;
        
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(true), Some(false));
            },
            _ => panic!("1 to many machine cycles in and_a_hl"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Bitwise XOR between the value in r8 and A.
     * 
     * MACHINE CYCLE: 1
     * INSTRUCTION LENGTH: 1
     */
    fn xor_a_r8(flag_reg: &mut u8, reg: u8, reg_a: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let result = reg ^ *reg_a;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(false));
            },
            _ => panic!("1 to many machine cycles in xor_a_r8"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Bitwise XOR between the byte pointed to by HL and A.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn xor_a_hl(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, reg_h: u8, reg_l: u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                let result = hl_data ^ *reg_a;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(false));
            }
            _ => panic!("1 to many machine cycles in xor_a_hl"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Bitwise OR between the value in r8 and A. 
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn or_a_r8(flag_reg: &mut u8, reg: u8, reg_a: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let result = reg | *reg_a;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(false));
            }, 
            _ => panic!("1 to many machine cycles in or_a_r8"),
        }
        return ExecuteStatus::Completed;

    }

    /**
    * Bitwise OR between the byte pointed to by HL and A.
    * 
    * MACHINE CYCLES: 2
    * INSTRUCTION LENGTH: 1
    */
    fn or_a_hl(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, reg_h: u8, reg_l: u8, machine_cycle: u8)  -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                let result = hl_data | *reg_a;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(false));
            },
            _ => panic!("1 to many machine cycles in or_a_hl"),
        }
        return ExecuteStatus::Completed;
    } 

    /**
     * Subtract the value in r8 from A and set flags accordingly, but don't store the result. This is useful for ComParing values.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1    
     */
    fn cp_a_r8(flag_reg: &mut u8, reg: u8, reg_a: u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let result = reg_a - reg;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some(reg > reg_a & 0xF), Some(reg > reg_a));
            },
            _ => panic!("1 to many machine cycles in cp_a_r8"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Subtract the byte pointed to by HL from A and set flags accordingly, but don't store the result.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn cp_a_hl(flag_reg: &mut u8, memory: &Memory, reg_a: u8, reg_h: u8, reg_l: u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let hl_data = memory.read_byte(binary_utils::build_16bit_num(reg_h, reg_l));
                let result = reg_a - hl_data;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some(hl_data > reg_a & 0xF), Some(hl_data > reg_a));
            },
            _ => panic!("1 to many machine cycles in cp_a_hl"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Return from subroutine if condition cc is met.
     * 
     * MACHINE CYCLES: 5 IF TAKEN/ 2 IF NOT TAKEN
     * INSTRUCTION LENGTH: 1
     */
    fn ret_cc(memory: &Memory, sp: &mut u16, pc: &mut u16, condition: bool, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                if !condition {
                    return ExecuteStatus::Completed;
                }
            },
            2 => { *temp_reg = memory.read_byte(*sp) as u16; *sp += 1; },         //Read lower SP byte
            3 => { *temp_reg |= (memory.read_byte(*sp) as u16) << 8; *sp += 1; }, //Read upper SP byte
            4 =>  {
                *pc = *temp_reg;
                return ExecuteStatus::Completed;
            }
            _ => panic!("1 to many machine cycles in ret_cc"),
        }
        return ExecuteStatus::Running;
    }

    /**
     * Pop to register r16 from the stack.
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 1
     */
    fn pop(memory: &Memory, upper_reg: &mut u8, lower_reg: &mut u8, sp: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => { 
                *lower_reg = memory.read_byte(*sp); 
                *sp += 1; 
            },
            2 => { 
                *upper_reg = memory.read_byte(*sp); 
                *sp += 1; 
                return ExecuteStatus::Completed;
            },
            _ => panic!("1 to many machine cycles in pop"),
        }
        return ExecuteStatus::Completed;
    }

    /**
    * Jump to address u16 if the condition is met
    * 
    * MACHINE CYCLES: 4
    * INSTRUCTION LENGTH: 3
    */
    fn jp_cc_u16(memory: &Memory, pc: &mut u16, condition: bool, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                *temp_reg = memory.read_byte(*pc) as u16;
                *pc += 1;
            },
            2 => {
                *temp_reg |= (memory.read_byte(*pc) as u16) << 8;
                *pc += 1;

                if !condition {
                    return ExecuteStatus::Completed;
                }
            },
            3 => {
                *pc = *temp_reg;
                return ExecuteStatus::Completed;
            },
            _ => panic!("1 to many machine cycles in jp_cc_u16"),
        }
        return ExecuteStatus::Running;
    }

    /**
    * Jump to address u16
    * 
    * MACHINE CYCLES: 4
    * INSTRUCTION LENGTH: 3
    */
    fn jp_u16(memory: &Memory, pc: &mut u16, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
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
                return ExecuteStatus::Completed;
            },
            _ => panic!("1 to many machine cycles in jp_u16"),
        }   

        return ExecuteStatus::Running;
    }

    /**
     * Call address u16 if condition cc is met.
     * 
     * MACHINE CYCLES: 6 IF TAKEN/ 3 IF NOT TAKEN
     * INSTRUCTION LENGTH: 3
     */
    fn call_cc_u16(memory: &mut Memory, pc: &mut u16, sp: &mut u16, condition: bool, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                *temp_reg = memory.read_byte(*pc) as u16;
                *pc += 1;
            },
            2 => {
                *temp_reg |= (memory.read_byte(*pc) as u16) << 8;
                *pc += 1;

                if !condition {
                    return ExecuteStatus::Completed;
                }
            },
            3 => {
                *sp -= 1;
                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(*pc); 
                memory.write_byte(*sp, upper_byte);
            },
            4 => {
                *sp -= 1;
                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(*pc); 
                memory.write_byte(*sp, lower_byte);
            },
            5 => {
                *pc = *temp_reg;
                return ExecuteStatus::Completed;
            },
            _ => panic!("1 to many machine cycles in call_cc_u16"),
        }     
        return ExecuteStatus::Running;
    }

    /**
     * Push register r16 into the stack
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 1
     */
    fn push_r16(memory: &mut Memory, upper_reg: u8, lower_reg: u8, sp: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => (), //No clue that this is doing at this cycle
            2 => {
                *sp -= 1;
                memory.write_byte(*sp, upper_reg);
            },
            3 => {
                *sp -= 1;
                memory.write_byte(*sp, lower_reg);
                return ExecuteStatus::Completed;
            },
            _ => panic!("1 to many machine cycles in push_r16"),
        }
        return ExecuteStatus::Running;
    }

    /**
     * Add the value u8 to A.
     * 
     * MACHINE CYCLE: 2
     * INSTRUCTION LENGTH: 2
     */
    fn add_a_u8(flag_reg: &mut u8, memory: &Memory, pc: &mut u16, reg_a: &mut u8, machine_cycle: u8) -> ExecuteStatus {
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
        return ExecuteStatus::Completed;
    }

    /**
     * Call address vec. This is a shorter and faster equivalent to CALL for suitable values of vec. Possibly not cycle accurate
     * 
     * MACHINE CYCLE: 4
     * INSTRUCTION LENGTH: 1
     */
    fn rst_vec(memory: &mut Memory, sp: &mut u16, pc: &mut u16, rst_address: u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                *sp -= 1;
                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(*pc);
                memory.write_byte(*sp, upper_byte);
            },
            2 => {
                *sp -= 1;
                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(*pc);
                memory.write_byte(*sp, lower_byte);
            },
            3 =>{
                *pc = rst_address;
                return ExecuteStatus::Completed;
            },
            _ => panic!("1 to many machine cycles in rst_vec"),
        }
        return ExecuteStatus::Running;
    }

    /**
     * Return from subroutine. This is basically a POP PC (if such an instruction existed). See POP r16 for an explanation of how POP works.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 1
     */
    fn ret(memory: &Memory, sp: &mut u16, pc: &mut u16, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
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
                return ExecuteStatus::Completed;
            }
            _ => panic!("1 to many machine cycles in ret"),
        }
        return ExecuteStatus::Running;
    }

    /**
    * Call address n16. This pushes the address of the instruction after the CALL on the stack, 
    * such that RET can pop it later; then, it executes an implicit JP n16.
    * 
    * MACHINE CYCLES: 6
    * INSTRUCTION LENGTH: 3
    */
    fn call_u16(memory: &mut Memory, pc: &mut u16, sp: &mut u16, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
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
                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(*pc);
                memory.write_byte(*sp, upper_byte);
            },
            4 => {
                *sp -= 1;
                let (upper_byte, lower_byte) = binary_utils::split_16bit_num(*pc);
                memory.write_byte(*sp, lower_byte);
            },
            5 => {
                *pc = *temp_reg;
                return ExecuteStatus::Completed;
            },
            _ => panic!("1 to many machine cycles in call_u16"),
        }
        return ExecuteStatus::Running;
    }

    /**
     * Add the value u8 plus the carry flag to A.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2 
     */
    fn adc_a_u8(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, pc: &mut u16, sp: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let value = memory.read_byte(*pc);
                *pc += 1;
        
                let (partial_result, first_overflow) = (*reg_a).overflowing_add(value);
                let (result, second_overflow) = partial_result.overflowing_add(Cpu::get_carry_flag(*flag_reg));
                let half_carry_overflow = (*reg_a & 0xF) + (value & 0xF) + Cpu::get_carry_flag(*flag_reg) > 0xF;
        
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(half_carry_overflow), Some(second_overflow));
            },
            _ => panic!("1 to many machine cycles in adc_a_u8"),
        }
        return ExecuteStatus::Completed;
    }

    /**
    * Subtract the value u8 from A.
    * 
    * MACHINE CYCLES: 2
    * INSTRUCTION LENGTH: 2
    */
    fn sub_a_u8(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, pc: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let value = memory.read_byte(*pc);
                *pc += 1;
        
                let result = *reg_a - value;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some(value > *reg_a & 0xF), Some(value > *reg_a));
            },
            _ => panic!("1 to many machine cycles in sub_a_u8"),
        }
    }

    /**
     * Return from subroutine and enable interrupts. 
     * This is basically equivalent to executing EI then RET, meaning that IME is set right after this instruction.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 1
     */
    fn reti(ime_flag: &mut bool, memory: &Memory, sp: &mut u16, pc: &mut u16, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
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
                *ime_flag = true;
                return ExecuteStatus::Completed;
            }
            _ => panic!("1 to many machine cycles in reti"),
        }
        todo!("Need to come back to this instruction as we need to delay the EI flag by 1 machine cycle. They say this is like EI then RET so I might just call those 2 functions");
        return ExecuteStatus::Running;
    }

    fn sbc_a_u8(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, pc: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let value = memory.read_byte(*pc);
                *pc += 1;
        
                let result = *reg_a - value - Cpu::get_carry_flag(*flag_reg);
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some((value - Cpu::get_carry_flag(*flag_reg)) > *reg_a & 0xF), Some(value + Cpu::get_carry_flag(*flag_reg) > *reg_a));
            },
            _ => panic!("1 to many machine cycles in sbc_a_u8"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Store value in register A into the byte at address $FF00+C.
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 2
     */
    fn ldh_u8_a(memory: &mut Memory, pc: &mut u16, reg_a: u8, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                *temp_reg = 0xFF00 | (memory.read_byte(*pc) as u16);
                *pc += 1;
            }
            2 => {
                memory.write_byte(*temp_reg, reg_a);
                return ExecuteStatus::Completed;
            }
            _ => panic!("1 to many machine cycles in ldh_u8_a"),
        }
        return ExecuteStatus::Running;
    }

    /**
    * Store value in register A into the byte at address $FF00+C.
    * 
    * MACHINE CYCLES: 2
    * INSTRUCTION LENGTH: 1
    */
    fn ldh_c_a(memory: &mut Memory, reg_a: u8, reg_c: u8, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                memory.write_byte(0xFF00 | reg_c as u16, reg_a);
            },
            _ => panic!("1 to many machine cycles in ldh_c_a"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Bitwise AND between u8 immediate and 8-bit A register
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION CYCLES: 2
     */
    fn and_a_u8(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, pc: &mut u16, machine_cycle: u8) -> ExecuteStatus {
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
        return ExecuteStatus::Completed;
    }

    /**
     * Add the 8-bit signed value i8 to 16-bit SP register.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 2
     */
    fn add_sp_i8(flag_reg: &mut u8, memory: &Memory,sp: &mut u16, pc: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle { 
            1 => {
                let value = memory.read_byte(*pc) as i8;
                *pc += 1;
        
                let result: u16 = (*sp).wrapping_add_signed(value);
                let half_carry_overflow = (*sp & 0xF) + (value & 0xF) > 0xF;
                let carry_overflow = (*sp & 0xFF) + (value & 0xFF) > 0xFF;
        
                *sp = result;
                Cpu::set_flags(flag_reg, Some(false), Some(false), Some(half_carry_overflow), Some(carry_overflow));
            },
            _ => panic!("1 to many machine cycles in add_sp_i8"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Jump to address in HL; effectively, load PC with value in register HL.
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn jp_hl(reg_h: u8, reg_l: u8, pc: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                *pc = binary_utils::build_16bit_num(reg_h, reg_l);
            },
            _ => panic!("1 to many machine cycles in jp_hl"),
        }
        return ExecuteStatus::Completed;
    }

    
    /**
     * Store value in register A into the byte at address u16.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 3
     */
    fn ld_u16_a(memory: &mut Memory, pc: &mut u16, reg_a: u8, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
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
                return ExecuteStatus::Completed;
            },
            _ => panic!("1 to many machine cycles in ld_u16_a"),
        }
        return ExecuteStatus::Running;
    }

    /**
     * Bitwise XOR between the value in u8 and A.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn xor_a_u8(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, pc: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let value = memory.read_byte(*pc);
                *pc += 1;
        
                let result = value ^ *reg_a;
                *reg_a = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(false));
            },
            _ => panic!("1 to many machine cycles in xor_a_u8"),
        }
        return ExecuteStatus::Completed;
    }

    fn ldh_a_u8(memory: &Memory, pc: &mut u16, reg_a: &mut u8, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                *temp_reg = 0xFF00 | (memory.read_byte(*pc) as u16);
                *pc += 1;
            },
            2 => {
                *reg_a = memory.read_byte(*temp_reg);
                return ExecuteStatus::Completed;
            },
            _ => panic!("1 to many machine cycles in ldh_a_u8"),
        }
        return ExecuteStatus::Running;
    }

    /**
     * Load value in register A from the byte at address $FF00+C.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ldh_a_c(memory: &Memory, reg_a: &mut u8, reg_c: u8, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                *reg_a = memory.read_byte(0xFF00 | reg_c as u16);
            },
            _ => panic!("1 to many machine cycles in ldh_a_c"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Disable Interrupts by clearing the IME flag. One reading mentions it cancels any scheduled effects of EI instruction
     * 
     * MACHINE CYCLES: 1
     * INSTRUCTION LENGTH: 1
     */
    fn di(ime_flag: &mut bool, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                *ime_flag = false;
            },
            _ => panic!("1 to many machine cycles in di"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Store into A the bitwise OR of u8 and A.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn or_a_u8(flag_reg: &mut u8, memory: &Memory, reg_a: &mut u8, pc: &mut u16, machine_cycle: u8) -> ExecuteStatus {
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
        return ExecuteStatus::Completed;
    }

    /**
     * Add the signed value i8 to SP and store the result in HL. CARRY HERE IS SO WRONG
     * 
     * MACHINE CYCLES: 3
     * INSTRUCTION LENGTH: 2
     */
    fn ld_hl_sp_i8(flag_reg: &mut u8, memory: &Memory, sp: &mut u16, pc: &mut u16, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let value = memory.read_byte(*pc) as i8;
                *pc += 1;

                let result: u16 = (*sp).wrapping_add_signed(value);
                let half_carry_overflow = (*sp & 0xF) + (value & 0xF) > 0xF;
                let carry_overflow = (*sp & 0xFF) + (value & 0xFF) > 0xFF;
        
                *sp = result;
                Cpu::set_flags(flag_reg, Some(false), Some(false), Some(half_carry_overflow), Some(carry_overflow));
            },
            2 => {
                *reg_l = (*sp & 0xFF) as u8;
                *reg_h = (*sp >> 8) as u8;
                return ExecuteStatus::Completed;
            },
            _ => panic!("1 to many machine cycles in ld_hl_sp_i8"),
        }
        return ExecuteStatus::Running;
    }

    /**
     * Load register HL into register SP.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 1
     */
    fn ld_sp_hl(reg_h: u8, reg_l: u8, sp: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                *sp = binary_utils::build_16bit_num(reg_h, reg_l);
            },
            _ => panic!("1 to many machine cycles in ld_sp_hl"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Load value in register A from the byte at address u16.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 3
     */
    fn ld_a_u16(memory: &Memory, pc: &mut u16, reg_a: &mut u8, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
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
                return ExecuteStatus::Completed;
            },
            _ => panic!("1 to many machine cycles in ld_a_u16"),
        }
        return ExecuteStatus::Running;
    }

    /**
     * Enable Interrupts by setting the IME flag. The flag is only set after the instruction following EI.\
     * 
     * MACHINE CYCLE: 1
     * INSTRUCTION LENGTH: 1
     */
    fn ei(ime_flag: &mut bool, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                *ime_flag = true;
            },
            _ => panic!("1 to many machine cycles in ei"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Subtract the value u8 from A and set flags accordingly, but don't store the result.
     * 
     * MACHINE CYCLE: 2
     * INSTRUCTION LENGTH: 2
     */
    fn cp_a_u8(flag_reg: &mut u8, memory: &Memory, reg_a: u8, pc: &mut u16, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let value = memory.read_byte(*pc);
                *pc += 1;
        
                let result = reg_a - value;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(true), Some(value > reg_a & 0xF), Some(value > reg_a));
            },
            _ => panic!("1 to many machine cycles in cp_a_u8"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * This will execute the instruction of the opcode on the prefix table. 
     * If the instruction is not complete it will return a Running status.
     */
    fn exexute_prefix(&mut self, memory: &mut Memory, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
        let opcode = memory.read_byte(self.pc);
        self.pc += 1; 

        match opcode {
            0x00 => Cpu::rlc_r8(&mut self.f, &mut self.b, machine_cycle),    //RLC B    
            0x01 => Cpu::rlc_r8(&mut self.f, &mut self.c, machine_cycle),    //RLC C 
            0x02 => Cpu::rlc_r8(&mut self.f, &mut self.d, machine_cycle),    //RLC D 
            0x03 => Cpu::rlc_r8(&mut self.f, &mut self.e, machine_cycle),    //RLC E 
            0x04 => Cpu::rlc_r8(&mut self.f, &mut self.h, machine_cycle),    //RLC H 
            0x05 => Cpu::rlc_r8(&mut self.f, &mut self.l, machine_cycle),    //RLC L
            0x06 => Cpu::rlc_hl(&mut self.f, memory, self.h, self.l, machine_cycle),         //RLC (HL)
            0x07 => Cpu::rlc_r8(&mut self.f, &mut self.a, machine_cycle),       //RLC A
            0x08 => Cpu::rrc_r8,
            /*0x09 => self.rrc_r8(Register::C),
            0x0A => self.rrc_r8(Register::D),
            0x0B => self.rrc_r8(Register::E),
            0x0C => self.rrc_r8(Register::H),
            0x0D => self.rrc_r8(Register::L),
            0x0E => self.rrc_hl(memory),
            0x0F => self.rrc_r8(Register::A),
            0x10 => self.rl_r8(Register::B),
            0x11 => self.rl_r8(Register::C),
            0x12 => self.rl_r8(Register::D),
            0x13 => self.rl_r8(Register::E),
            0x14 => self.rl_r8(Register::H),
            0x15 => self.rl_r8(Register::L),
            0x16 => self.rl_hl(memory),
            0x17 => self.rl_r8(Register::A),
            0x18 => self.rr_r8(Register::B),
            0x19 => self.rr_r8(Register::C),
            0x1A => self.rr_r8(Register::D),
            0x1B => self.rr_r8(Register::E),
            0x1C => self.rr_r8(Register::H),
            0x1D => self.rr_r8(Register::L),
            0x1E => self.rr_hl(memory),
            0x1F => self.rr_r8(Register::A),
            0x20 => self.sla_r8(Register::B),
            0x21 => self.sla_r8(Register::C),
            0x22 => self.sla_r8(Register::D),
            0x23 => self.sla_r8(Register::E),
            0x24 => self.sla_r8(Register::H),
            0x25 => self.sla_r8(Register::L),
            0x26 => self.sla_hl(memory),
            0x27 => self.sla_r8(Register::A),
            0x28 => self.sra_r8(Register::B),
            0x29 => self.sra_r8(Register::C),
            0x2A => self.sra_r8(Register::D),
            0x2B => self.sra_r8(Register::E),
            0x2C => self.sra_r8(Register::H),
            0x2D => self.sra_r8(Register::L),
            0x2E => self.sra_hl(memory),
            0x2F => self.sra_r8(Register::A),
            0x30 => self.swap_r8(Register::B),
            0x31 => self.swap_r8(Register::C),
            0x32 => self.swap_r8(Register::D),
            0x33 => self.swap_r8(Register::E),
            0x34 => self.swap_r8(Register::H),
            0x35 => self.swap_r8(Register::L),
            0x36 => self.swap_hl(memory),
            0x37 => self.swap_r8(Register::A),
            0x38 => self.srl_r8(Register::B),
            0x39 => self.srl_r8(Register::B),
            0x3A => self.srl_r8(Register::B),
            0x3B => self.srl_r8(Register::B),
            0x3C => self.srl_r8(Register::B),
            0x3D => self.srl_r8(Register::B),
            0x3E => self.srl_hl(memory),
            0x3F => self.srl_r8(Register::A),
            0x40 => self.bit_u3_r8(0, Register::B),
            0x41 => self.bit_u3_r8(0, Register::C),
            0x42 => self.bit_u3_r8(0, Register::D),
            0x43 => self.bit_u3_r8(0, Register::E),
            0x44 => self.bit_u3_r8(0, Register::H),
            0x45 => self.bit_u3_r8(0, Register::L),
            0x46 => self.bit_u3_hl(0, memory),
            0x47 => self.bit_u3_r8(0, Register::A),
            0x48 => self.bit_u3_r8(1, Register::B),
            0x49 => self.bit_u3_r8(1, Register::C),
            0x4A => self.bit_u3_r8(1, Register::D),
            0x4B => self.bit_u3_r8(1, Register::E),
            0x4C => self.bit_u3_r8(1, Register::H),
            0x4D => self.bit_u3_r8(1, Register::L),
            0x4E => self.bit_u3_hl(1, memory),
            0x4F => self.bit_u3_r8(1, Register::A),
            0x50 => self.bit_u3_r8(2, Register::B),
            0x51 => self.bit_u3_r8(2, Register::C),
            0x52 => self.bit_u3_r8(2, Register::D),
            0x53 => self.bit_u3_r8(2, Register::E),
            0x54 => self.bit_u3_r8(2, Register::H),
            0x55 => self.bit_u3_r8(2, Register::L),
            0x56 => self.bit_u3_hl(2, memory),
            0x57 => self.bit_u3_r8(2, Register::A),
            0x58 => self.bit_u3_r8(3, Register::B),
            0x59 => self.bit_u3_r8(3, Register::C),
            0x5A => self.bit_u3_r8(3, Register::D),
            0x5B => self.bit_u3_r8(3, Register::E),
            0x5C => self.bit_u3_r8(3, Register::H),
            0x5D => self.bit_u3_r8(3, Register::L),
            0x5E => self.bit_u3_hl(3, memory),
            0x5F => self.bit_u3_r8(3, Register::A),
            0x60 => self.bit_u3_r8(4, Register::B),
            0x61 => self.bit_u3_r8(4, Register::C),
            0x62 => self.bit_u3_r8(4, Register::D),
            0x63 => self.bit_u3_r8(4, Register::E),
            0x64 => self.bit_u3_r8(4, Register::H),
            0x65 => self.bit_u3_r8(4, Register::L),
            0x66 => self.bit_u3_hl(4, memory),
            0x67 => self.bit_u3_r8(4, Register::A),
            0x68 => self.bit_u3_r8(5, Register::B),
            0x69 => self.bit_u3_r8(5, Register::C),
            0x6A => self.bit_u3_r8(5, Register::D),
            0x6B => self.bit_u3_r8(5, Register::E),
            0x6C => self.bit_u3_r8(5, Register::H),
            0x6D => self.bit_u3_r8(5, Register::L),
            0x6E => self.bit_u3_hl(5, memory),
            0x6F => self.bit_u3_r8(5, Register::A),
            0x70 => self.bit_u3_r8(6, Register::B),
            0x71 => self.bit_u3_r8(6, Register::C),
            0x72 => self.bit_u3_r8(6, Register::D),
            0x73 => self.bit_u3_r8(6, Register::E),
            0x74 => self.bit_u3_r8(6, Register::H),
            0x75 => self.bit_u3_r8(6, Register::L),
            0x76 => self.bit_u3_hl(6, memory),
            0x77 => self.bit_u3_r8(6, Register::A),
            0x78 => self.bit_u3_r8(7, Register::B),
            0x79 => self.bit_u3_r8(7, Register::C),
            0x7A => self.bit_u3_r8(7, Register::D),
            0x7B => self.bit_u3_r8(7, Register::E),
            0x7C => self.bit_u3_r8(7, Register::H),
            0x7D => self.bit_u3_r8(7, Register::L),
            0x7E => self.bit_u3_hl(7, memory),
            0x7F => self.bit_u3_r8(7, Register::A),
            0x80 => self.res_u3_r8(0, Register::B),
            0x81 => self.res_u3_r8(0, Register::C),
            0x82 => self.res_u3_r8(0, Register::D),
            0x83 => self.res_u3_r8(0, Register::E),
            0x84 => self.res_u3_r8(0, Register::H),
            0x85 => self.res_u3_r8(0, Register::L),
            0x86 => self.res_u3_hl(0, memory),
            0x87 => self.res_u3_r8(0, Register::A),
            0x88 => self.res_u3_r8(1, Register::B),
            0x89 => self.res_u3_r8(1, Register::C),
            0x8A => self.res_u3_r8(1, Register::D),
            0x8B => self.res_u3_r8(1, Register::E),
            0x8C => self.res_u3_r8(1, Register::H),
            0x8D => self.res_u3_r8(1, Register::L),
            0x8E => self.res_u3_hl(1, memory),
            0x8F => self.res_u3_r8(1, Register::A),
            0x90 => self.res_u3_r8(2, Register::B),
            0x91 => self.res_u3_r8(2, Register::C),
            0x92 => self.res_u3_r8(2, Register::D),
            0x93 => self.res_u3_r8(2, Register::E),
            0x94 => self.res_u3_r8(2, Register::H),
            0x95 => self.res_u3_r8(2, Register::L),
            0x96 => self.res_u3_hl(2, memory),
            0x97 => self.res_u3_r8(2, Register::A),
            0x98 => self.res_u3_r8(3, Register::B),
            0x99 => self.res_u3_r8(3, Register::C),
            0x9A => self.res_u3_r8(3, Register::D),
            0x9B => self.res_u3_r8(3, Register::E),
            0x9C => self.res_u3_r8(3, Register::H),
            0x9D => self.res_u3_r8(3, Register::L),
            0x9E => self.res_u3_hl(3, memory),
            0x9F => self.res_u3_r8(3, Register::A),
            0xA0 => self.res_u3_r8(4, Register::B),
            0xA1 => self.res_u3_r8(4, Register::C),
            0xA2 => self.res_u3_r8(4, Register::D),
            0xA3 => self.res_u3_r8(4, Register::E),
            0xA4 => self.res_u3_r8(4, Register::H),
            0xA5 => self.res_u3_r8(4, Register::L),
            0xA6 => self.res_u3_hl(4, memory),
            0xA7 => self.res_u3_r8(4, Register::A),
            0xA8 => self.res_u3_r8(5, Register::B),
            0xA9 => self.res_u3_r8(5, Register::C),
            0xAA => self.res_u3_r8(5, Register::D),
            0xAB => self.res_u3_r8(5, Register::E),
            0xAC => self.res_u3_r8(5, Register::H),
            0xAD => self.res_u3_r8(5, Register::L),
            0xAE => self.res_u3_hl(5, memory),
            0xAF => self.res_u3_r8(5, Register::A),
            0xB0 => self.res_u3_r8(6, Register::B),
            0xB1 => self.res_u3_r8(6, Register::C),
            0xB2 => self.res_u3_r8(6, Register::D),
            0xB3 => self.res_u3_r8(6, Register::E),
            0xB4 => self.res_u3_r8(6, Register::H),
            0xB5 => self.res_u3_r8(6, Register::L),
            0xB6 => self.res_u3_hl(6, memory),
            0xB7 => self.res_u3_r8(6, Register::A),
            0xB8 => self.res_u3_r8(7, Register::B),
            0xB9 => self.res_u3_r8(7, Register::C),
            0xBA => self.res_u3_r8(7, Register::D),
            0xBB => self.res_u3_r8(7, Register::E),
            0xBC => self.res_u3_r8(7, Register::H),
            0xBD => self.res_u3_r8(7, Register::L),
            0xBE => self.res_u3_hl(7, memory),
            0xBF => self.res_u3_r8(7, Register::A),
            0xC0 => self.set_u3_r8(0, Register::B),
            0xC1 => self.set_u3_r8(0, Register::C),
            0xC2 => self.set_u3_r8(0, Register::D),
            0xC3 => self.set_u3_r8(0, Register::E),
            0xC4 => self.set_u3_r8(0, Register::H),
            0xC5 => self.set_u3_r8(0, Register::L),
            0xC6 => self.set_u3_hl(0, memory),
            0xC7 => self.set_u3_r8(0, Register::A),
            0xC8 => self.set_u3_r8(1, Register::B),
            0xC9 => self.set_u3_r8(1, Register::C),
            0xCA => self.set_u3_r8(1, Register::D),
            0xCB => self.set_u3_r8(1, Register::E),
            0xCC => self.set_u3_r8(1, Register::H),
            0xCD => self.set_u3_r8(1, Register::L),
            0xCE => self.set_u3_hl(1, memory),
            0xCF => self.set_u3_r8(1, Register::A),
            0xD0 => self.set_u3_r8(2, Register::B),
            0xD1 => self.set_u3_r8(2, Register::C),
            0xD2 => self.set_u3_r8(2, Register::D),
            0xD3 => self.set_u3_r8(2, Register::E),
            0xD4 => self.set_u3_r8(2, Register::H),
            0xD5 => self.set_u3_r8(2, Register::L),
            0xD6 => self.set_u3_hl(2, memory),
            0xD7 => self.set_u3_r8(2, Register::A),
            0xD8 => self.set_u3_r8(3, Register::B),
            0xD9 => self.set_u3_r8(3, Register::C),
            0xDA => self.set_u3_r8(3, Register::D),
            0xDB => self.set_u3_r8(3, Register::E),
            0xDC => self.set_u3_r8(3, Register::H),
            0xDD => self.set_u3_r8(3, Register::L),
            0xDE => self.set_u3_hl(3, memory),
            0xDF => self.set_u3_r8(3, Register::A),
            0xE0 => self.set_u3_r8(4, Register::B),
            0xE1 => self.set_u3_r8(4, Register::C),
            0xE2 => self.set_u3_r8(4, Register::D),
            0xE3 => self.set_u3_r8(4, Register::E),
            0xE4 => self.set_u3_r8(4, Register::H),
            0xE5 => self.set_u3_r8(4, Register::L),
            0xE6 => self.set_u3_hl(4, memory),
            0xE7 => self.set_u3_r8(4, Register::A),
            0xE8 => self.set_u3_r8(5, Register::B),
            0xE9 => self.set_u3_r8(5, Register::C),
            0xEA => self.set_u3_r8(5, Register::D),
            0xEB => self.set_u3_r8(5, Register::E),
            0xEC => self.set_u3_r8(5, Register::H),
            0xED => self.set_u3_r8(5, Register::L),
            0xEE => self.set_u3_hl(5, memory),
            0xEF => self.set_u3_r8(5, Register::A),
            0xF0 => self.set_u3_r8(6, Register::B),
            0xF1 => self.set_u3_r8(6, Register::C),
            0xF2 => self.set_u3_r8(6, Register::D),
            0xF3 => self.set_u3_r8(6, Register::E),
            0xF4 => self.set_u3_r8(6, Register::H),
            0xF5 => self.set_u3_r8(6, Register::L),
            0xF6 => self.set_u3_hl(6, memory),
            0xF7 => self.set_u3_r8(6, Register::A),
            0xF8 => self.set_u3_r8(7, Register::B),
            0xF9 => self.set_u3_r8(7, Register::C),
            0xFA => self.set_u3_r8(7, Register::D),
            0xFB => self.set_u3_r8(7, Register::E),
            0xFC => self.set_u3_r8(7, Register::H),
            0xFD => self.set_u3_r8(7, Register::L),
            0xFE => self.set_u3_hl(7, memory),
            0xFF => self.set_u3_r8(7, Register::A),
            */
            _ => panic!("opcode is not recognized"),
        }
    }

    /**
     * Rotate register r8 left.
     * 
     * MACHINE CYCLES: 2
     * INSTRUCTION LENGTH: 2
     */
    fn rlc_r8(flag_reg: &mut u8, reg: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let result = (*reg).rotate_left(1);
                *reg = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(*reg & 0x80 == 0x80));
            },
            _ => panic!("1 to many machine cycles in rlc_r8"),
        }
        return ExecuteStatus::Completed;
    }

    /**
     * Rotate the byte pointed to by HL left.
     * 
     * MACHINE CYCLES: 4
     * INSTRUCTION LENGTH: 2
     */
    fn rlc_hl(flag_reg: &mut u8, memory: &Memory, reg_h: u8, reg_l: u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => (), //Should be reading from hl here but nah
            2 => {
                let value = memory.read_byte(build_16bit_num(reg_h, reg_l));
                let result = value.rotate_left(1);
                memory.write_byte(build_16bit_num(reg_h, reg_l), result);
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(result & 0x80 == 0x80));
                return ExecuteStatus::Completed;
            },
            _ => panic!("1 to many machine cycles in rlc_hl"),
        }
        return ExecuteStatus::Running;
    }

    /**
    * Rotate register r8 right.
    * 
    * MACHINE CYCLE: 2
    * INSTRUCTION LENGTH: 2
    */
    fn rrc_r8(flag_reg: &mut u8, reg: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let result = (*reg).rotate_right(1);
                *reg = result;
                Cpu::set_flags(flag_reg, Some(result == 0), Some(false), Some(false), Some(*reg & 0x80 == 0x80));
            },
            _ => panic!("1 to many machine cycles in rrc_r8"),
        }
        return ExecuteStatus::Completed;
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

