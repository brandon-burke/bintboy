use std::{fs::File, io::{Read, Seek, SeekFrom}};
pub struct Rom {
    pub banks: Vec<[u8; 0x4000]>
}

impl Rom {
    pub fn new() -> Self {
        Self { 
            banks: vec![] 
        }
    }

    /**
     * Takes a file path to a Game Boy rom file and loads it into the rom struct.
     * This will separate the rom into 16KB banks.
     */
    pub fn load_rom(&mut self, file_path: &str) {
        let mut rom_file = File::open(file_path).expect("File not found");
        let file_size = rom_file.seek(SeekFrom::End(0)).expect("Error finding the file size");
        let num_of_banks = file_size / 0x4000;
        println!("{file_size} {num_of_banks}");
        rom_file.seek(SeekFrom::Start(0)).expect("Error resetting the file");

        //Setting up how many 16KB banks the rom has
        for _ in 0..num_of_banks {
            self.banks.push([0; 0x4000]);
        }

        let mut byte_count = 0;
        let mut bank_num = 0;
        for byte in rom_file.bytes() {
            if byte_count < 0x4000 {            
                self.banks[bank_num][byte_count] = match byte {
                    Ok(byte_value) => byte_value,
                    Err(e) => panic!("Error reading rom on bank: {bank_num} and byte: {byte_count}\n {e}"),
                };

                byte_count += 1;
            } else {
                byte_count = 0;
                bank_num += 1;
            }
        }
    }
}