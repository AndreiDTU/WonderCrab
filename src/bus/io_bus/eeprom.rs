pub struct EEPROM {
    pub contents: Vec<u8>,
    input: u16,
    output: u16,

    comm: u16,
    address_bits: u8,

    write_enabled: bool,
}

impl EEPROM {
    pub fn new(contents: Vec<u8>, address_bits: u8) -> Self {
        Self {
            contents,
            input: 0, output: 0,

            comm: 0, address_bits,

            write_enabled: true,
        }
    }

    pub fn read_data(&self) -> u16 {
        self.output
    }

    pub fn write_data(&mut self, data: u16) {
        self.input = data;
    }

    pub fn write_comm(&mut self, comm: u16) {
        if comm >> (self.address_bits + 3) != 0 {return}
        self.comm = comm;

        let sb = (comm >> (self.address_bits + 2)) & 1 != 0;
        if sb {
            // println!("Running op!");
            let opcode = (comm >> self.address_bits) & 3;
            if opcode == 0 {
                let sub_op = (comm >> (self.address_bits - 2)) & 3;
                self.execute_sub_op(sub_op as u8);
            } else {
                let address = (comm & ((1 << self.address_bits) - 1)) * 2;
                // if opcode == 2 {println!("EEPROM contents at {:04X} = {:04X}", address, u16::from_le_bytes([self.contents[address as usize], self.contents[address as usize + 1]]))}
                // println!("Address {:04X}", address);
                self.execute_op(address, opcode as u8);
                // println!("Output = {:04X}", self.output);
            }
        }
    }

    fn execute_op(&mut self, address: u16, opcode: u8) {
        match opcode {
            // WRITE
            1 => {
                if self.write_enabled {
                    // println!("EEPROM [{:04X}] <- {:04X}", address, self.input);
                    let bytes = self.input.to_le_bytes();
                    self.contents[address as usize]     = bytes[0];
                    self.contents[address as usize + 1] = bytes[1];
                    // println!("EEPROM [{:04X}] = {:04X}", address, u16::from_le_bytes([self.contents[address as usize], self.contents[address as usize + 1]]));
                }
            }
            // READ
            2 => self.output = u16::from_le_bytes([self.contents[address as usize], self.contents[address as usize + 1]]),
            // ERASE
            3 => {
                if self.write_enabled {
                    self.contents[address as usize]     = 0xFF;
                    self.contents[address as usize + 1] = 0xFF;
                }
            }
            _ => unreachable!()
        }
    }

    fn execute_sub_op(&mut self, opcode: u8) {
        match opcode {
            // EWDS
            0 => self.write_enabled = false,
            // WRAL
            1 => if self.write_enabled {
                self.contents = self.contents.iter().enumerate().map(|(i, _)| {
                    let bytes = self.input.to_le_bytes();
                    if i % 2 == 0 {
                        bytes[0]
                    } else {
                        bytes[1]
                    }
                }).collect();
            }
            // ERAL
            2 => if self.write_enabled {self.contents.fill(0xFF)},
            // EWEN
            3 => self.write_enabled = true,
            _ => unreachable!()
        }
    }
}