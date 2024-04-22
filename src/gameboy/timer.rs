const MACHINE_CYCLE: u8 = 4;

pub struct Timer {
    div_reg: u16,               // Divider Register
    tima_reg: u8,               // Timer Counter
    tma_reg: u8,                // Timer Modulo
    tac_reg: u8,                // Timer Control
    prev_div_bit_value: u8,     // 
    tima_overflow: bool,        // TIMA overflow      
    div_write: bool,            //Lets us know if a write to the div register occurred
    tac_en_falling_edge: bool,  //Lets us know if the timer went from being enabled to disabled
    ticks_since_overflow: u8,   //
    pub interrupted_requested: bool,
    tima_a_cycle_write_occurred: bool,
    tima_write_value: u8,
}

impl Timer {
    /**
     * This is going to create a Timer and return it
     */
    pub fn new() -> Timer {
        Timer {
            div_reg: 0xABCC,
            tima_reg: 0,
            tma_reg: 0,
            tac_reg: 0xF8,
            prev_div_bit_value: 0,
            tima_overflow: false,
            div_write: false,
            tac_en_falling_edge: false,
            ticks_since_overflow: 0,
            interrupted_requested: false,
            tima_a_cycle_write_occurred: false,
            tima_write_value: 0,
        }
    }

    /**
     * Mimicking one cpu clk cycle. NOT a machine cycle, which is 4 cpu clk cycles
     */
    pub fn cycle(&mut self) {
        self.increment_div();
        if ((self.falling_edge_detected() && self.timer_is_enabled()) || (self.get_current_div_bit_value() == 1 && self.tac_en_falling_edge)) && !self.tima_overflow {
            self.increment_tima();
        }

        if self.tima_overflow == true {
            self.ticks_since_overflow += 1; 
            
            if self.ticks_since_overflow == MACHINE_CYCLE && self.tima_a_cycle_write_occurred { //The A cycle
                self.tima_reg = self.tima_write_value;
                self.tima_overflow = false;
                self.ticks_since_overflow = 0;
                self.tima_a_cycle_write_occurred = false;
            } else if self.ticks_since_overflow == 8 {   //B cycle
                self.tima_reg = self.tma_reg; 
                self.interrupted_requested = true;
                self.ticks_since_overflow = 0;
                self.tima_overflow = false;
            }
        }       
        
        self.prev_div_bit_value = self.get_current_div_bit_value(); //At the end of the cycle we'll update what the previous bit value was
    }

    /**
     * Increments tima, will update a flag if the value overflowed, and request an interrupt
     */
    fn increment_tima(&mut self) {
        let (result, overflowed) = self.tima_reg.overflowing_add(1);
        self.tima_reg = result;
        self.tima_overflow = overflowed;
    }

    fn increment_div(&mut self) {
        if self.div_write {
            self.div_reg = 0;
            self.div_write = false;
        } else {
            self.div_reg = self.div_reg.wrapping_add(1);
        }
    }

    /**
     * This function will return true if the selected bit in the div register
     * transitions from a 1 to a 0. The bit to look at is determined by bit 0 and 
     * bit 1 of the TAC register
     */
    fn falling_edge_detected(&self) -> bool {
        if self.prev_div_bit_value == 1 && self.get_current_div_bit_value() == 0 {
            return true;
        }
        return false;
    }

    /**
     * Return the value of whatever bit we were looking at in the DIV register
     */
    fn get_current_div_bit_value(&self) -> u8 {
        let current_clk_selction = self.tac_reg & 0x3;
        match current_clk_selction {
            0x00 => ((self.div_reg >> 9) & 0x1) as u8,
            0x01 => ((self.div_reg >> 3) & 0x1) as u8,
            0x02 => ((self.div_reg >> 5) & 0x1) as u8,
            0x03 => ((self.div_reg >> 7) & 0x1) as u8,
            _ => panic!("reading an unknown clk frequency"),
        }
    }

    fn timer_is_enabled(&self) -> bool {
        let timer_enable_bit = (self.tac_reg >> 2) & 0x1;
        if timer_enable_bit == 0 {
            return false;
        }
        return true;
    }

    pub fn write_2_div(&mut self) {
        self.div_write = true;
    }
    
    pub fn write_2_tima(&mut self, value: u8) {
        if self.ticks_since_overflow <= MACHINE_CYCLE {
            if self.tima_overflow {
                self.tima_a_cycle_write_occurred = true;
                self.tima_write_value = value;
            } else {
                self.tima_reg = value;
            }
        }
    }
    
    pub fn write_2_tma(&mut self, value: u8) { 
        self.tma_reg = value;
    }

    pub fn write_2_tac(&mut self, value: u8) {
        let old_tac_enable_bit = self.timer_is_enabled();
        self.tac_reg = (self.tac_reg & 0xF8) | (value & 0x7);
        let new_tac_enable_bit = self.timer_is_enabled();

        if old_tac_enable_bit == true && new_tac_enable_bit == false {
            self.tac_en_falling_edge = true;
        }
    }

    /**
     * Returning whats in the div register
     */
    pub fn read_div(&self) -> u8 {
        return (self.div_reg >> 8) as u8;
    }

    pub fn read_tima(&self) -> u8 {
        return self.tima_reg;
    }

    pub fn read_tma(&self) -> u8 {
        return self.tma_reg;
    }

    pub fn read_tac(&self) -> u8 {
        return self.tac_reg;
    }
}