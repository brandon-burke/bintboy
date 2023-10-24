use crate::timer::Timer;
use crate::interrupt_handler::InterruptHandler;
use crate::constants::*;

pub struct Memory {
    rom_bank_0: [u8; 0x4000],   //16KB -> 0000h – 3FFFh (Non-switchable ROM bank)
    rom_bank_x: [u8; 0x4000],   //16KB -> 4000h – 7FFFh (Switchable ROM bank)
    vram: [u8; 0x2000],         //8KB  -> 8000h – 9FFFh (Video RAM)
    sram: [u8; 0x2000],         //8KB  -> A000h – BFFFh (External RAM in cartridge)
    wram_0: [u8; 0x1000],       //1KB  -> C000h – CFFFh (Work RAM)
    wram_x: [u8; 0x1000],       //1KB  -> D000h – DFFFh (Work RAM)
    echo: [u8; 0x1E00],         //     -> E000h – FDFFh (ECHO RAM) Mirror of C000h-DDFFh
    oam: [u8; 0xA0],            //     -> FE00h – FE9Fh (Object Attribute Table) Sprite information table
    unused: [u8; 0x60],         //     -> FEA0h – FEFFh (Unused)
    io: [u8; 0x80],             //     -> FF00h – FF7Fh (I/O ports)
    pub interrupt_handler: InterruptHandler, //Will contain IE, IF, and IME registers (0xFFFF, 0xFF0F)
    timer: Timer,               //     -> FF04 - FF07
    hram: [u8; 0x7F],           //     -> FF80h – FFFEh (HRAM)
    ie_reg: [u8; 0x1],          //     -> FFFFh         (Interrupt enable flags)
}

impl Memory {
    pub fn new() -> Self {
        Self {
            rom_bank_0: [0; 0x4000],   
            rom_bank_x: [0; 0x4000],   
            vram: [0; 0x2000],         
            sram: [0; 0x2000],        
            wram_0: [0; 0x1000],       
            wram_x: [0; 0x1000],   
            echo: [0; 0x1E00],        
            oam: [0; 0xA0],            
            unused: [0; 0x60],
            io: [0; 0x80],
            interrupt_handler: InterruptHandler::new(), 
            timer: Timer::new(),                 
            hram: [0; 0x7F],           
            ie_reg: [0; 0x1],    
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_START ..= ROM_BANK_0_END => self.rom_bank_0[address as usize],
            ROM_BANK_X_START ..= ROM_BANK_X_END => self.rom_bank_x[(address - ROM_BANK_X_START) as usize],
            VRAM_START ..= VRAM_END => self.vram[(address - VRAM_START) as usize],
            SRAM_START ..= SRAM_END => self.sram[(address - SRAM_START) as usize],
            WRAM_0_START ..= WRAM_0_END => self.wram_0[(address - WRAM_0_START) as usize],
            WRAM_X_START ..= WRAM_X_END => self.wram_x[(address - WRAM_X_START) as usize],
            ECHO_START ..= ECHO_END => {
                panic!("I don't think we should be accessing echo memory");
            }
            UNUSED_START ..= UNUSED_END => {
                panic!("I don't think we should be accessing unused memory");
            }
            IO_START ..= IO_END => {
                match address {
                    TIMER_DIV_REG => self.timer.read_div(),
                    TIMER_TIMA_REG => self.timer.read_tima(),
                    TIMER_TMA_REG => self.timer.read_tma(),
                    TIMER_TAC_REG => self.timer.read_tac(),   
                    INTERRUPT_FLAG_REG => self.interrupt_handler.read_if_reg(),
                    _ => self.io[(address - IO_START) as usize],
                } 
            }
            HRAM_START ..= HRAM_END => self.hram[(address - HRAM_START) as usize],
            INTERRUPT_ENABLE_START => self.interrupt_handler.read_ie_reg(),
            _ => panic!("MEMORY ACCESS OUT OF BOUNDS"),
        } 
    }

    pub fn write_byte(&mut self, address: u16, data_to_write: u8) {
        match address {
            ROM_BANK_0_START ..= ROM_BANK_0_END => self.rom_bank_0[address as usize] = data_to_write,
            ROM_BANK_X_START ..= ROM_BANK_X_END => self.rom_bank_x[(address - ROM_BANK_X_START) as usize] = data_to_write,
            VRAM_START ..= VRAM_END => self.vram[(address - VRAM_START) as usize] = data_to_write,
            SRAM_START ..= SRAM_END => self.sram[(address - SRAM_START) as usize] = data_to_write,
            WRAM_0_START ..= WRAM_0_END => self.wram_0[(address - WRAM_0_START) as usize] = data_to_write,
            WRAM_X_START ..= WRAM_X_END => self.wram_x[(address - WRAM_X_START) as usize] = data_to_write,
            ECHO_START ..= ECHO_END => {
                panic!("I don't think we should be writing echo memory");
            }
            UNUSED_START ..= UNUSED_END => {
                panic!("I don't think we should be writing unused memory");
            }
            IO_START ..= IO_END => {
                match address {
                    TIMER_DIV_REG => self.timer.write_2_div(),
                    TIMER_TIMA_REG => self.timer.write_2_tima(data_to_write),
                    TIMER_TMA_REG => self.timer.write_2_tma(data_to_write),
                    TIMER_TAC_REG => self.timer.write_2_tac(data_to_write),
                    INTERRUPT_FLAG_REG => self.interrupt_handler.write_if_reg(data_to_write),             
                    _ => self.io[(address - IO_START) as usize] = data_to_write,
                }
            }
            HRAM_START ..= HRAM_END => self.hram[(address - HRAM_START) as usize] = data_to_write,
            INTERRUPT_ENABLE_START => self.interrupt_handler.write_ie_reg(data_to_write),
            _ => panic!("MEMORY ACCESS OUT OF BOUNDS"),
        } 
    }

    pub fn timer_cycle(&mut self) {
        self.timer.cycle();
        if self.timer.interrupted_requested {
            self.interrupt_handler.if_reg |= 0x04;
            self.timer.interrupted_requested = false;
        }
    }

    /**
     * Again this is wildly ugly but I had to pull out the memory read and writes because they aren't avaliable
     * to the interrupt handler, but its also apart of the memory object so I can't pass it in. I'm going to have
     * to refactor this later
     */
    pub fn interrupt_cycle(&mut self, pc: &mut u16, sp: &mut u16) {
        match self.interrupt_handler.cycle(pc) {
            3 => {
                *sp = (*sp).wrapping_sub(1);
                self.write_byte(*sp, (*pc >> 8) as u8);
            },
            4 => {
                *sp = (*sp).wrapping_sub(1);
                self.write_byte(*sp, *pc as u8);
            },
            _ => (), 
        }
    }

    pub fn load_rom(&mut self, rom_0: [u8; 0x4000], rom_1: [u8; 0x4000]) {
        self.rom_bank_0 = rom_0;
        self.rom_bank_x = rom_1;
    }
}