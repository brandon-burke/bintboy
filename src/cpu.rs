use crate::memory::Memory;
use crate::cpu_state::{CpuState, ExecuteStatus};
use crate::opcodes::{OPCODE_MACHINE_CYCLES, PREFIX_OPCODE_MACHINE_CYCLES};
use crate::binary_utils;

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
            0x09 => Cpu::add_hl_r16(&mut self.f, &mut self.b, &mut self.c, &mut self.h, &mut self.l, machine_cycle),  //ADD_HL_BC may not be the most cycle accurate
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
            0x19 => Cpu::add_hl_r16(&mut self.f, &mut self.d, &mut self.e, &mut self.h, &mut self.l, machine_cycle),    //ADD_HL_DE
            0x1A => Cpu::ld_a_r16(memory, &mut self.a, self.d, self.e, machine_cycle),                  //LD_A_R16
            0x1B => Cpu::dec_r16(&mut self.d, &mut self.e, machine_cycle),                              //DEC_DE
            0x1C => Cpu::inc_r8(&mut self.f, &mut self.e, machine_cycle),                               //INC_E
            0x1D => Cpu::dec_r8(&mut self.f, &mut self.e, machine_cycle),                               //DEC_E
            0x1E => Cpu::ld_r8_u8(memory, &mut self.e, &mut self.pc, machine_cycle),                              //LD_E_U8
            0x1F => Cpu::rra(&mut self.f, &mut self.a, machine_cycle),                                  //RRA
            0x20 => Cpu::jr_cc_i8(memory, &mut self.pc, !Cpu::get_zero_flag(self.f), machine_cycle, temp_reg)   ,           //JR_NZ_I8                                        
            0x21 => Cpu::ld_r16_u16(memory, &mut self.h, &mut self.l, &mut self.pc, machine_cycle),       //LD_HL_U16
            0x22 => Cpu::ld_hli_a(memory),
             /*0x23 => self.inc_r16(Register::H, Register::L),
            0x24 => self.inc_r8(Register::H),
            0x25 => self.dec_r8(Register::H),
            0x26 => self.ld_r8_u8(memory, Register::H),
            0x27 => self.daa(),
            0x28 => self.jr_cc_i8(memory, self.get_zero_flag()),
            0x29 => self.add_hl_r16(Register::H, Register::L),
            0x2A => self.ld_a_hli(memory),
            0x2B => self.dec_r16(Register::H, Register::L),
            0x2C => self.inc_r8(Register::L),
            0x2D => self.dec_r8(Register::L),
            0x2E => self.ld_r8_u8(memory, Register::L),
            0x2F => self.cpl(),
            0x30 => self.jr_cc_i8(memory, !self.get_carry_flag()),
            0x31 => self.ld_sp_u16(memory),
            0x32 => self.ld_hld_a(memory),
            0x33 => self.inc_sp(),
            0x34 => self.inc_hl(memory),
            0x35 => self.dec_hl(memory),
            0x36 => self.ld_hl_u8(memory),
            0x37 => self.scf(),
            0x38 => self.jr_cc_i8(memory, self.get_carry_flag()),
            0x39 => self.add_hl_sp(),
            0x3A => self.ld_a_hld(memory),
            0x3B => self.dec_sp(),
            0x3C => self.inc_r8(Register::A),
            0x3D => self.dec_r8(Register::A),
            0x3E => self.ld_r8_u8(memory, Register::A),
            0x3F => self.ccf(),
            0x40 => self.ld_r8_r8(Register::B, Register::B),
            0x41 => self.ld_r8_r8(Register::B, Register::C),
            0x42 => self.ld_r8_r8(Register::B, Register::D),
            0x43 => self.ld_r8_r8(Register::B, Register::E),
            0x44 => self.ld_r8_r8(Register::B, Register::H),
            0x45 => self.ld_r8_r8(Register::B, Register::L),
            0x46 => self.ld_r8_hl(memory, Register::B),
            0x47 => self.ld_r8_r8(Register::B, Register::A),
            0x48 => self.ld_r8_r8(Register::C, Register::B),
            0x49 => self.ld_r8_r8(Register::C, Register::C),
            0x4A => self.ld_r8_r8(Register::C, Register::D),
            0x4B => self.ld_r8_r8(Register::C, Register::E),
            0x4C => self.ld_r8_r8(Register::C, Register::H),
            0x4D => self.ld_r8_r8(Register::C, Register::L),
            0x4E => self.ld_r8_hl(memory, Register::C),
            0x4F => self.ld_r8_r8(Register::C, Register::A),
            0x50 => self.ld_r8_r8(Register::D, Register::B),
            0x51 => self.ld_r8_r8(Register::D, Register::C),
            0x52 => self.ld_r8_r8(Register::D, Register::D),
            0x53 => self.ld_r8_r8(Register::D, Register::E),
            0x54 => self.ld_r8_r8(Register::D, Register::H),
            0x55 => self.ld_r8_r8(Register::D, Register::L),
            0x56 => self.ld_r8_hl(memory, Register::D),
            0x57 => self.ld_r8_r8(Register::D, Register::A),
            0x58 => self.ld_r8_r8(Register::E, Register::B),
            0x59 => self.ld_r8_r8(Register::E, Register::C),
            0x5A => self.ld_r8_r8(Register::E, Register::D),
            0x5B => self.ld_r8_r8(Register::E, Register::E),
            0x5C => self.ld_r8_r8(Register::E, Register::H),
            0x5D => self.ld_r8_r8(Register::E, Register::L),
            0x5E => self.ld_r8_hl(memory, Register::E),
            0x5F => self.ld_r8_r8(Register::E, Register::A),
            0x60 => self.ld_r8_r8(Register::H, Register::B),
            0x61 => self.ld_r8_r8(Register::H, Register::C),
            0x62 => self.ld_r8_r8(Register::H, Register::D),
            0x63 => self.ld_r8_r8(Register::H, Register::E),
            0x64 => self.ld_r8_r8(Register::H, Register::H),
            0x65 => self.ld_r8_r8(Register::H, Register::L),
            0x66 => self.ld_r8_hl(memory, Register::H),
            0x67 => self.ld_r8_r8(Register::H, Register::A),
            0x68 => self.ld_r8_r8(Register::L, Register::B),
            0x69 => self.ld_r8_r8(Register::L, Register::C),
            0x6A => self.ld_r8_r8(Register::L, Register::D),
            0x6B => self.ld_r8_r8(Register::L, Register::E),
            0x6C => self.ld_r8_r8(Register::L, Register::H),
            0x6D => self.ld_r8_r8(Register::L, Register::L),
            0x6E => self.ld_r8_hl(memory, Register::L),
            0x6F => self.ld_r8_r8(Register::L, Register::A),
            0x70 => self.ld_hl_r8(memory, Register::B),
            0x71 => self.ld_hl_r8(memory, Register::C),
            0x72 => self.ld_hl_r8(memory, Register::D),
            0x73 => self.ld_hl_r8(memory, Register::E),
            0x74 => self.ld_hl_r8(memory, Register::H),
            0x75 => self.ld_hl_r8(memory, Register::L),
            0x76 => self.halt(),
            0x77 => self.ld_hl_r8(memory, Register::A),
            0x78 => self.ld_r8_r8(Register::A, Register::B),
            0x79 => self.ld_r8_r8(Register::A, Register::C),
            0x7A => self.ld_r8_r8(Register::A, Register::D),
            0x7B => self.ld_r8_r8(Register::A, Register::E),
            0x7C => self.ld_r8_r8(Register::A, Register::H),
            0x7D => self.ld_r8_r8(Register::A, Register::L),
            0x7E => self.ld_r8_hl(memory, Register::A), 
            0x7F => self.ld_r8_r8(Register::A, Register::A),
            0x80 => self.add_a_r8(Register::B),
            0x81 => self.add_a_r8(Register::C),
            0x82 => self.add_a_r8(Register::D),
            0x83 => self.add_a_r8(Register::E),
            0x84 => self.add_a_r8(Register::H),
            0x85 => self.add_a_r8(Register::L),
            0x86 => self.add_a_hl(memory),
            0x87 => self.add_a_r8(Register::A),
            0x88 => self.adc_a_r8(Register::B),
            0x89 => self.adc_a_r8(Register::C),
            0x8A => self.adc_a_r8(Register::D),
            0x8B => self.adc_a_r8(Register::E),
            0x8C => self.adc_a_r8(Register::H),
            0x8D => self.adc_a_r8(Register::L),
            0x8E => self.adc_a_hl(memory),
            0x8F => self.adc_a_r8(Register::A),
            0x90 => self.sub_a_r8(Register::B),
            0x91 => self.sub_a_r8(Register::C),
            0x92 => self.sub_a_r8(Register::D),
            0x93 => self.sub_a_r8(Register::E),
            0x94 => self.sub_a_r8(Register::H),
            0x95 => self.sub_a_r8(Register::L),
            0x96 => self.sub_a_hl(memory),
            0x97 => self.sub_a_r8(Register::A),
            0x98 => self.sbc_a_r8(Register::B),
            0x99 => self.sbc_a_r8(Register::C),
            0x9A => self.sbc_a_r8(Register::D),
            0x9B => self.sbc_a_r8(Register::E),
            0x9C => self.sbc_a_r8(Register::H),
            0x9D => self.sbc_a_r8(Register::L),
            0x9E => self.sbc_a_hl(memory),
            0x9F => self.sbc_a_r8(Register::A),
            0xA0 => self.and_a_r8(Register::B),
            0xA1 => self.and_a_r8(Register::C),
            0xA2 => self.and_a_r8(Register::D),
            0xA3 => self.and_a_r8(Register::E),
            0xA4 => self.and_a_r8(Register::H),
            0xA5 => self.and_a_r8(Register::L),
            0xA6 => self.and_a_hl(memory),
            0xA7 => self.and_a_r8(Register::A),
            0xA8 => self.xor_a_r8(Register::B),
            0xA9 => self.xor_a_r8(Register::C),
            0xAA => self.xor_a_r8(Register::D),
            0xAB => self.xor_a_r8(Register::E),
            0xAC => self.xor_a_r8(Register::H),
            0xAD => self.xor_a_r8(Register::L),
            0xAE => self.xor_a_hl(memory),
            0xAF => self.xor_a_r8(Register::A),
            0xB0 => self.or_a_r8(Register::B),
            0xB1 => self.or_a_r8(Register::C),
            0xB2 => self.or_a_r8(Register::D),
            0xB3 => self.or_a_r8(Register::E),
            0xB4 => self.or_a_r8(Register::H),
            0xB5 => self.or_a_r8(Register::L),
            0xB6 => self.or_a_hl(memory),
            0xB7 => self.or_a_r8(Register::A),
            0xB8 => self.cp_a_r8(Register::B),
            0xB9 => self.cp_a_r8(Register::C),
            0xBA => self.cp_a_r8(Register::D),
            0xBB => self.cp_a_r8(Register::E),
            0xBC => self.cp_a_r8(Register::H),
            0xBD => self.cp_a_r8(Register::L),
            0xBE => self.cp_a_hl(memory),
            0xBF => self.cp_a_r8(Register::A),
            0xC0 => self.ret_cc(memory, !self.get_zero_flag()),
            0xC1 => self.pop(memory, Register::B, Register::C),
            0xC2 => self.jp_cc_u16(memory, !self.get_zero_flag()),
            0xC3 => self.jp_u16(memory),
            0xC4 => self.call_cc_u16(memory, !self.get_zero_flag()),
            0xC5 => self.push_r16(memory, Register::B, Register::C),
            0xC6 => self.add_a_u8(memory),
            0xC7 => self.rst_vec(memory, 0x00),
            0xC8 => self.ret_cc(memory, self.get_zero_flag()),
            0xC9 => self.ret(memory),
            0xCA => self.jp_cc_u16(memory, self.get_zero_flag()),
            0xCB => self.prefix(memory),    //Need to pass the self.current opcode here as that will
            0xCC => self.call_cc_u16(memory, self.get_zero_flag()),
            0xCD => self.call_u16(memory),
            0xCE => self.adc_a_u8(memory),
            0xCF => self.rst_vec(memory, 0x08),
            0xD0 => self.ret_cc(memory, !self.get_carry_flag()),
            0xD1 => self.pop(memory, Register::D, Register::E),
            0xD2 => self.jp_cc_u16(memory, !self.get_carry_flag()),
            0xD3 => panic!("0xD3 is an unused opcode"),
            0xD4 => self.call_cc_u16(memory, !self.get_carry_flag()),
            0xD5 => self.push_r16(memory, Register::D, Register::E),
            0xD6 => self.sub_a_u8(memory),
            0xD7 => self.rst_vec(memory, 0x10),
            0xD8 => self.ret_cc(memory, self.get_carry_flag()),
            0xD9 => self.reti(memory),
            0xDA => self.jp_cc_u16(memory, self.get_carry_flag()),
            0xDB => panic!("0xDB is an unused opcode"),
            0xDC => self.call_cc_u16(memory, self.get_carry_flag()),
            0xDE => panic!("0xDE is an unused opcode"),
            0xDF => self.rst_vec(memory, 0x18),
            0xE0 => self.ldh_u8_a(memory),
            0xE1 => self.pop(memory, Register::H, Register::L),
            0xE2 => self.ldh_c_a(memory),
            0xE3 => panic!("0xE3 is an unused opcode"),
            0xE4 => panic!("0xE4 is an unused opcode"),
            0xE5 => self.push_r16(memory, Register::H, Register::L),
            0xE6 => self.and_a_u8(memory),
            0xE7 => self.rst_vec(memory, 0x20),
            0xE8 => self.add_sp_i8(memory),
            0xE9 => self.jp_hl(),
            0xEA => self.ld_u16_a(memory),
            0xEB => panic!("0xEB is an unused opcode"),
            0xEC => panic!("0xEC is an unused opcode"),
            0xED => panic!("0xED is an unused opcode"),
            0xEE => self.xor_a_u8(memory),
            0xEF => self.rst_vec(memory, 0x28),
            0xF0 => self.ldh_a_u8(memory),
            0xF1 => self.pop(memory, Register::A, Register::F),
            0xF2 => self.ldh_a_c(memory),
            0xF3 => self.di(memory),
            0xF4 => panic!("0xF4 is an unused opcode"),
            0xF5 => self.push_r16(memory, Register::A, Register::F),
            0xF6 => self.or_a_u8(memory),
            0xF7 => self.rst_vec(memory, 0x30),
            0xF8 => self.ld_hl_sp_i8(memory),
            0xF9 => self.ld_sp_hl(),
            0xFA => self.ld_a_u16(memory),
            0xFB => self.ei(memory),
            0xFC => panic!("0xFC is an unused opcode"),
            0xFD => panic!("0xFD is an unused opcode"),
            0xFE => self.cp_a_u8(memory),
            0xFF => self.rst_vec(memory, 0x38),
            */
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

    pub fn get_carry_flag(flag_reg: u8) -> u8 {
        binary_utils::get_bit(flag_reg, 4)
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
        let mut status = ExecuteStatus::Running;
        match machine_cycle {
            1 => *lower_reg = memory.read_byte(*pc),
            2 => { 
                *upper_reg = memory.read_byte(*pc); 
                status = ExecuteStatus::Completed; 
            },
            _ => panic!("1 to many cycles on ld_r16_u16"),
        }
        *pc += 1;
        return status;
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
    fn add_hl_r16(flag_reg: &mut u8, upper_reg: &mut u8, lower_reg: &mut u8, reg_h: &mut u8, reg_l: &mut u8, machine_cycle: u8) -> ExecuteStatus {
        match machine_cycle {
            1 => {
                let reg_16 = binary_utils::build_16bit_num(*upper_reg, *lower_reg);
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
    pub fn ld_hli_a(memory: &mut Memory, machine_cycle: u8) {
        
        memory.write_byte(self.hl(), self.a);
        self.set_hl(self.hl() + 1);
    }

















    fn get_zero_flag(flag_reg: u8) -> bool {
        ((flag_reg >> 7) & 0x1) != 0
    }












    /**
    * Given an opcode it will execute the instruction of the opcode. If 
    */
    pub fn exexute_prefix(&mut self, memory: &mut Memory, machine_cycle: u8, temp_reg: &mut u16) -> ExecuteStatus {
        match self.current_opcode {
            _ => ExecuteStatus::Error,
        }
    }
}

