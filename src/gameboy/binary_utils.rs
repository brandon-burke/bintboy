pub fn get_bit(value: u8, bit_position: u8) -> u8 {
    return (value >> bit_position) & 0x1;
}

pub fn get_bit_16(value: u16, bit_position: u8) -> u16 {
    return (value >> bit_position) & 0x1;
}

pub fn build_16bit_num(upper_byte: u8, lower_byte: u8) -> u16 {
    return (upper_byte as u16) << 8 | lower_byte as u16;
}

pub fn split_16bit_num(value: u16) -> (u8, u8) {
    let upper_byte = (value >> 8) as u8;
    let lower_byte = value as u8;

    return (upper_byte, lower_byte);
}

pub fn set_bit(value: u8, bit_position: u8) -> u8 {
    return value | (0x1 << bit_position);
}

pub fn reset_bit(value: u8, bit_position: u8) -> u8 {
    return value & !(0x1 << bit_position);
}