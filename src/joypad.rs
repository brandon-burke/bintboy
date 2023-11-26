use crate::binary_utils;
enum State {
    On,
    Off,
}

impl State {
    fn is_off(&self) -> bool {
        match self {
            State::On => false,
            State::Off => true,
        }
    }

    fn value(&self) -> u8 {
        match self {
            State::On => 0,
            State::Off => 1,
        }
    }

    /**
     * This is going to return a State enum depending if it received 
     * a 1 or 0. Any other numbers will cause an error
     */
    fn convert_from_num(num: u8) -> Self {
        match num {
            0 => State::On,
            1 => State::Off,
            _ => panic!("Cannot convert {} to a State enum. Must be 0 or 1", num),
        }
    }
}

pub struct Joypad {
    a_and_right: State,     //READ-ONLY
    b_and_left: State,      //READ-ONLY
    select_and_up: State,   //READ-ONLY
    start_and_down: State,  //READ-ONLY
    select_dpad: State, 
    select_buttons: State,
    unused_bit_6: State,
    unused_bit_7: State,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            a_and_right: State::Off, 
            b_and_left: State::Off,
            select_and_up: State::Off,
            start_and_down: State::Off,
            select_dpad: State::Off,
            select_buttons: State::Off,
            unused_bit_6: State::Off,
            unused_bit_7: State::Off,
        }
    }

    /**
     * Returns the joypad values packed into a byte of data. NOTE that 
     * if neither the d-pad or buttons are selected, then the lower
     * nibble will be 0xF
     */
    pub fn read_joypad_reg(&self) -> u8 {
        let upper_nibble = (self.unused_bit_7.value() << 7) | 
                            (self.unused_bit_6.value() << 6) | 
                            (self.select_buttons.value() << 5) | 
                            (self.select_dpad.value() << 4);

        if self.select_buttons.is_off() && self.select_dpad.is_off() {
            return upper_nibble | 0xF;
        }

        return  upper_nibble | 
                (self.start_and_down.value() << 3) | 
                (self.select_and_up.value() << 2) |
                (self.b_and_left.value() << 1) |
                (self.a_and_right.value());
    }

    /**
     * Unpacking each bit and writing the value to the
     * corresponding field. The lower nibble is READ-ONLY
     */
    pub fn write_joypad_reg(&mut self, data_to_write: u8) {
        self.select_dpad = State::convert_from_num(binary_utils::get_bit(data_to_write, 4));
        self.select_buttons = State::convert_from_num(binary_utils::get_bit(data_to_write, 5));
        self.unused_bit_6 = State::convert_from_num(binary_utils::get_bit(data_to_write, 6));
        self.unused_bit_7 = State::convert_from_num(binary_utils::get_bit(data_to_write, 7));
    }
}
