use crate::cpu::v30mz::V30MZ;

pub struct SoC {
    // COMPONENTS
    cpu: V30MZ,

    // MEMORY
    wram: [u8; 0xFFFF],

    // I/O
    io: [u8; 0xFF],
}

pub trait MemBus {
    fn read_mem(&mut self, addr: u32) -> Result<u8, ()>;
    fn write_mem(&mut self, addr: u32, byte: u8);

    fn read_mem_16(&mut self, addr: u32) -> Result<u16, ()>{
        let bytes = [self.read_mem(addr)?, self.read_mem(addr.wrapping_add(1))?];
        Ok(u16::from_le_bytes(bytes))
    }

    fn write_mem_16(&mut self, addr: u32, src: u16) {
        let bytes = src.to_le_bytes();
        self.write_mem(addr, bytes[0]);
        self.write_mem(addr.wrapping_add(1), bytes[1]);
    }

    fn read_mem_32(&mut self, addr: u32) -> Result<(u16, u16), ()>{
        let bytes1 = [self.read_mem(addr)?, self.read_mem(addr.wrapping_add(1))?];
        let bytes2 = [self.read_mem(addr.wrapping_add(2))?, self.read_mem(addr.wrapping_add(3))?];
        let result1 = u16::from_le_bytes(bytes1);
        let result2 = u16::from_le_bytes(bytes2);
        Ok((result1, result2))
    }
}

pub trait IOBus {
    fn read_io(&mut self, addr: u16) -> Result<u8, ()>;
    fn write_io(&mut self, addr: u16, byte: u8);

    fn read_io_16(&mut self, addr: u16) -> Result<(u8, u8), ()>{
        Ok((self.read_io(addr)?, self.read_io(addr.wrapping_add(1))?))
    }

    fn write_io_16(&mut self, addr: u16, byte: u8) {
        self.write_io(addr, byte);
        self.write_io(addr.wrapping_add(1), byte);
    }
}

impl MemBus for SoC {
    fn read_mem(&mut self, addr: u32) -> Result<u8, ()> {
        let byte = match addr {
            0x00000..=0x03FFF => self.wram[addr as usize],
            0x04000..=0x0FFFF => {
                if !self.color_mode() {
                    0x90
                } else {
                    self.wram[addr as usize]
                }
            }
            addr => panic!("Not yet implemented! Addr: {:05X}", addr)
        };

        Ok(byte)
    } 

    fn write_mem(&mut self, addr: u32, data: u8) {
        match addr {
            0x00000..=0x03FFF => self.wram[addr as usize] = data,
            0x04000..=0x0FFFF => if self.color_mode() {
                self.wram[addr as usize] = data;
            }
            _ => todo!()
        }
    }
}

impl IOBus for SoC {
    fn read_io(&mut self, addr: u16) -> Result<u8, ()> {
        if addr & 0x0100 != 0 {
            return Ok(0x90)
        }

        let port = addr as u8;
        if addr > 0xFF && port > 0xB8 {
            return Ok(0x90)
        }

        Ok(self.io[port as usize])
    }
    
    fn write_io(&mut self, addr: u16, byte: u8) {
        self.io[addr as usize] = byte
    }
}

impl SoC {
    pub fn new() -> Self {
        Self {
            cpu: V30MZ::new(),

            wram: [0; 0xFFFF],

            io: [0; 0xFF],
        }
    }

    pub fn tick(&mut self) {
        self.cpu.tick();

        loop {
            // Supply program bytes
            if self.cpu.op_request {
                let pc = self.cpu.get_pc_address();
                for addr in pc..pc.wrapping_add(8) {
                    let byte = self.read_mem(addr).unwrap();
                    self.cpu.current_op.push(byte);
                }
                let _ = self.cpu.execute();
                continue;
            }

            // I/O Requests
            if !self.cpu.io_read_requests.is_empty() {
                for addr in self.cpu.io_read_requests.clone() {
                    let byte = self.read_io(addr).unwrap();
                    self.cpu.io_responses.insert(addr, byte);
                }
                self.cpu.io_read_requests.clear();
                let _ = self.cpu.execute();
                continue;
            }

            if !self.cpu.read_requests.is_empty() {
                for addr in self.cpu.read_requests.clone() {
                    let byte = self.read_mem(addr).unwrap();
                    self.cpu.read_responses.insert(addr, byte);
                }
                self.cpu.read_requests.clear();
                let _ = self.cpu.execute();
                continue;
            }

            break;
        }

        if !self.cpu.write_requests.is_empty() {
            for (addr, byte) in self.cpu.write_requests.clone() {
                self.write_mem(addr, byte);
            }
        }
    }

    fn color_mode(&mut self) -> bool {
        self.read_io(0x60).unwrap() >> 7 != 0
    }
}

#[cfg(test)]
pub mod test;