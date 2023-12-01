use crate::binary_utils;

enum ClockSpeed {
    NormalSpeed,
    DoubleSpeed,
}

impl ClockSpeed {
    fn value(&self) -> u8 {
        match self {
            ClockSpeed::NormalSpeed => 0,
            ClockSpeed::DoubleSpeed => 1,
        }
    }

    fn convert_from_num(num: u8) -> Self {
        match num {
            0 => ClockSpeed::NormalSpeed,
            1 => ClockSpeed::DoubleSpeed,
            _ => panic!("Cannot convert {} to a ClockSpeed. Must be 0 or 1", num),
        }
    }
}

enum ClockSelect {
    Slave,
    Master,
}

impl ClockSelect {
    fn value(&self) -> u8 {
        match self {
            ClockSelect::Slave => 0,
            ClockSelect::Master => 1,
        }
    }

    fn convert_from_num(num: u8) -> Self {
        match num {
            0 => ClockSelect::Slave,
            1 => ClockSelect::Master,
            _ => panic!("Cannot convert {} to a ClockSelect. Must be 0 or 1", num),
        }
    }
}
enum TransferStatus {
    Idle,
    RequestedOrInProgress,
}

impl TransferStatus {
    fn value(&self) -> u8 {
        match self {
            TransferStatus::Idle => 0,
            TransferStatus::RequestedOrInProgress => 1,
        }
    }

    fn convert_from_num(num: u8) -> Self {
        match num {
            0 => TransferStatus::Idle,
            1 => TransferStatus::RequestedOrInProgress,
            _ => panic!("Cannot convert {} to a TransferStatus. Must be 0 or 1", num),
        }
    }
}
pub struct SerialTransfer {
    sb: u8,       //$FF01 Serial Transfer register
    transfer_enable: TransferStatus,
    unused_bit_6: u8,
    unused_bit_5: u8,
    unused_bit_4: u8,
    unused_bit_3: u8,
    unused_bit_2: u8,
    clock_speed: ClockSpeed,        //CGB Feature
    clock_select: ClockSelect,
}

impl SerialTransfer {
    pub fn new() -> Self {
        Self {
            sb: 0,
            transfer_enable: TransferStatus::Idle,
            unused_bit_6: 0,
            unused_bit_5: 0,
            unused_bit_4: 0,
            unused_bit_3: 0,
            unused_bit_2: 0,
            clock_speed: ClockSpeed::NormalSpeed,
            clock_select: ClockSelect::Master,
        }
    }
    /**
     * Returns what is in address $FF01
     */
    pub fn read_sb_reg(&self) -> u8 {
        return self.sb;
    }

    /**
     * Returns what is in address $FF02
     */
    pub fn read_sc_reg(&self) -> u8 {
        return (self.transfer_enable.value() << 7) |
                (self.unused_bit_6 << 6) |
                (self.unused_bit_5 << 5) |
                (self.unused_bit_4 << 4) |
                (self.unused_bit_3 << 3) |
                (self.unused_bit_2 << 2) |
                (self.clock_speed.value() << 1) |
                (self.clock_select.value())
    }

    pub fn write_sb_reg(&mut self, data_to_write: u8) {
        self.sb = data_to_write;
    }

    pub fn write_sc_reg(&mut self, data_to_write: u8) {
        self.transfer_enable = TransferStatus::convert_from_num(binary_utils::get_bit(data_to_write, 7));
        self.unused_bit_6 = binary_utils::get_bit(data_to_write, 6);
        self.unused_bit_5 = binary_utils::get_bit(data_to_write, 5);
        self.unused_bit_4 = binary_utils::get_bit(data_to_write, 4);
        self.unused_bit_3 = binary_utils::get_bit(data_to_write, 3);
        self.unused_bit_2 = binary_utils::get_bit(data_to_write, 2);
        self.clock_speed = ClockSpeed::convert_from_num(binary_utils::get_bit(data_to_write, 1));
        self.clock_select = ClockSelect::convert_from_num(binary_utils::get_bit(data_to_write, 0));
    }
}

