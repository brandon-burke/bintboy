enum State {
    On = 0,
    Off = 1,
}

impl State {
    fn value(&self) -> u8 {
        match self {
            State::On => 0,
            State::Off => 1,
        }
    }
}

struct Joypad {
    empty_bit_7: State,
    empty_bit_6: State,
    select_buttons: State,
    select_dpad: State,
    start_and_down: State,
    select_and_up: State,
    b_and_left: State,
    a_and_right: State,
}

impl Joypad {
    fn new() -> Self {
        Self {
            empty_bit_7: State::Off,
            empty_bit_6: State::Off,
            select_buttons: State::Off,
            select_dpad: State::Off,
            start_and_down: State::Off,
            select_and_up: State::Off,
            b_and_left: State::Off,
            a_and_right: State::Off, 
        }
    }

    /**
     * Returns the joypad values packed into a byte of data
     */
    fn read_joypad_reg(&self) -> u8 {
        let reg = (self.empty_bit_7.value() << 7) | (self.empty_bit_6.value() << 6) | (self.select_buttons.value() << 5) | 
            (self.select_dpad.value() << 4) | (self.start_and_down << 3) | (self.select_dpad.value() << 2)
    }

    fn write_joypad_reg(&mut self) {
        
    }
}
