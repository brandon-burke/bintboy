pub struct Dma {
    src_address_reg: u8,            //$FF46 in memory
    current_address_offset: u8,     //Will help keep track of what address we are currently reading from and to
    cycles_since_start: u8,         //Tells us how many cycles have passed since the start of the DMA transfer
    clk_ticks_before_write: u8,        //Will help us write every 4 cpu clks
    pub currently_transferring: bool    //Tells us if we are currently transferring a some data
}

impl Dma {
    pub fn new() -> Self {
        Dma {
            src_address_reg: 0,
            current_address_offset: 0,
            cycles_since_start: 0,
            clk_ticks_before_write: 0,
            currently_transferring: false,
        }
    }

    /**
     * This function carries out 1 cpu clk cycle. We either return nothing or
     * something which contains the src address were reading from and the offset 
     * for the OAM address we are writing to. We only return something if we
     * are currently transferring and if 4 cpu cycles have passed since the last
     * OAM write
     */
    pub fn cycle(&mut self) -> Option<(u16, u8)> {
        if self.currently_transferring {
            self.clk_ticks_before_write += 1;
            
            if self.clk_ticks_before_write == 4 {
                self.clk_ticks_before_write = 0;

                //Checking if were finished transferring
                self.cycles_since_start += 1;
                if self.cycles_since_start == 160 {
                    self.currently_transferring = false;
                }
                
                let src_address: u16 = ((self.src_address_reg as u16) << 8) + self.current_address_offset as u16;
                self.current_address_offset += 1;
                return Some((src_address, self.current_address_offset - 1)) ;
            }
        }
        return None;
    }

    pub fn read_source_address(&self) -> u8 {
        return self.src_address_reg;
    }

    /**
     * If we write to DMA during an active transfer then we cancel the current
     * transfer 
     */
    pub fn write_source_address(&mut self, value: u8) {
        self.src_address_reg = value;
        self.current_address_offset = 0;
        self.clk_ticks_before_write = 0;
        self.cycles_since_start = 0;
        self.currently_transferring = true;
    }
}