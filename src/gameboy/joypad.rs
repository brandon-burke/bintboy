use minifb::{Key, Window};
use crate::gameboy::binary_utils;

#[derive(Copy, Clone)]
enum ButtonState {
    On,
    Off,
}

impl ButtonState {
    fn is_off(&self) -> bool {
        match self {
            ButtonState::On => false,
            ButtonState::Off => true,
        }
    }

    fn is_on(&self) -> bool {
        match self {
            ButtonState::On => true,
            ButtonState::Off => false,
        }
    }

    fn value(&self) -> u8 {
        match self {
            ButtonState::On => 0,
            ButtonState::Off => 1,
        }
    }

    /**
     * This is going to return a ButtonState enum depending if it received
     * a 1 or 0. Any other numbers will cause an error
     */
    fn convert_from_num(num: u8) -> Self {
        match num {
            0 => ButtonState::On,
            1 => ButtonState::Off,
            _ => panic!("Cannot convert {} to a ButtonState enum. Must be 0 or 1", num),
        }
    }
}

pub struct Joypad {
    a_and_right: ButtonState,     //READ-ONLY
    b_and_left: ButtonState,      //READ-ONLY
    select_and_up: ButtonState,   //READ-ONLY
    start_and_down: ButtonState,  //READ-ONLY
    select_dpad: ButtonState,
    select_buttons: ButtonState,
    unused_bit_6: ButtonState,
    unused_bit_7: ButtonState,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            a_and_right: ButtonState::Off,
            b_and_left: ButtonState::Off,
            select_and_up: ButtonState::Off,
            start_and_down: ButtonState::Off,
            select_dpad: ButtonState::Off,
            select_buttons: ButtonState::Off,
            unused_bit_6: ButtonState::Off,
            unused_bit_7: ButtonState::Off,
        }
    }

    //Essentially just need to check the keys in questions every cycle. If they change state going from
    //High to low then they have been pressed
    pub fn cycle(&mut self, window: &Window) -> bool {
        let mut request_interrupt = false;
        let mut prev_button_state;

        //Capturing the old state and updating to new state
        prev_button_state = self.a_and_right;
        self.a_and_right = match (window.is_key_down(Key::J) && self.select_buttons.is_on()) | (window.is_key_down(Key::D) && self.select_dpad.is_on()) {
            true => ButtonState::On,
            false => ButtonState::Off,
        };
        //Checking if the button is pressed and if its considered active
        if self.select_dpad.is_on() || self.select_buttons.is_on() {
            request_interrupt = match (prev_button_state, self.a_and_right) {
                (ButtonState::Off, ButtonState::On) => {
                    true
                }
                _ => request_interrupt,
            };
        }

        prev_button_state = self.b_and_left;
        self.b_and_left = match (window.is_key_down(Key::K) && self.select_buttons.is_on()) | (window.is_key_down(Key::A) && self.select_dpad.is_on()) {
            true => ButtonState::On,
            false => ButtonState::Off,
        };
        //Checking if the button is pressed and if its considered active
        if self.select_dpad.is_on() || self.select_buttons.is_on() {
            request_interrupt = match (prev_button_state, self.b_and_left) {
                (ButtonState::Off, ButtonState::On) => {
                    true
                }
                _ => request_interrupt,
            };
        }

        prev_button_state = self.select_and_up;
        self.select_and_up = match (window.is_key_down(Key::W) && self.select_buttons.is_on()) | (window.is_key_down(Key::Backspace) && self.select_dpad.is_on()) {
            true => ButtonState::On,
            false => ButtonState::Off
        };
        //Checking if the button is pressed and if its considered active
        if self.select_dpad.is_on() || self.select_buttons.is_on() {
            request_interrupt = match (prev_button_state, self.select_and_up) {
                (ButtonState::Off, ButtonState::On) => {
                    true
                }
                _ => request_interrupt,
            };
        }
        
        prev_button_state = self.start_and_down;
        self.start_and_down = match (window.is_key_down(Key::Space) && self.select_buttons.is_on()) | (window.is_key_down(Key::S) && self.select_dpad.is_on()) {
            true => ButtonState::On,
            false => ButtonState::Off,
        };
        //Checking if the button is pressed and if its considered active
        if self.select_dpad.is_on() || self.select_buttons.is_on() {
            request_interrupt = match (prev_button_state, self.start_and_down) {
                (ButtonState::Off, ButtonState::On) => {
                    true
                }
                _ => request_interrupt,
            };
        }

        
        return request_interrupt;
    }

    /*
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
        self.select_dpad = ButtonState::convert_from_num(binary_utils::get_bit(data_to_write, 4));
        self.select_buttons = ButtonState::convert_from_num(binary_utils::get_bit(data_to_write, 5));
        self.unused_bit_6 = ButtonState::convert_from_num(binary_utils::get_bit(data_to_write, 6));
        self.unused_bit_7 = ButtonState::convert_from_num(binary_utils::get_bit(data_to_write, 7));
    }
}
