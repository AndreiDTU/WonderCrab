use crate::cpu::v30mz::V30MZ;

pub struct SoC {
    // COMPONENTS
    cpu: V30MZ,

    // MEMORY
    wram: [u8; 0xFFFF],

    // I/O
    io_in: [u8; 0xFF],
}

impl SoC {
    pub fn new() -> Self {
        Self {
            cpu: V30MZ::new(),

            wram: [0; 0xFFFF],

            io_in: [0; 0xFF],
        }
    }

    pub fn tick(&mut self) {
        self.cpu.tick();

        loop {
            // Supply program bytes
            if self.cpu.op_request {
                let addr = self.cpu.get_pc_address();
                let byte = self.read_mem(addr);
                self.cpu.current_op.push(byte);
                let _ = self.cpu.execute();
                continue;
            }

            // I/O Requests
            if self.cpu.io_request {
                let addr = self.cpu.get_io_address();
                let byte = self.read_io(addr);
                self.cpu.io_response.push(byte);
                let _ = self.cpu.execute();
                continue;
            }

            if !self.cpu.read_requests.is_empty() {
                for addr in self.cpu.read_requests.clone() {
                    let byte = self.read_mem(addr);
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

    pub fn read_mem(&mut self, addr: u32) -> u8 {
        match addr {
            0x00000..=0x03FFF => self.wram[addr as usize],
            0x04000..=0x0FFFF => {
                return if !self.color_mode() {
                    0x90
                } else {
                    self.wram[addr as usize]
                }
            }
            _ => todo!()
        }
    }

    pub fn write_mem(&mut self, addr: u32, data: u8) {
        match addr {
            0x00000..=0x03FFF => self.wram[addr as usize] = data,
            0x04000..=0x0FFFF => if self.color_mode() {
                self.wram[addr as usize] = data;
            }
            _ => todo!()
        }
    }

    pub fn read_io(&self, addr: u16) -> u8 {
        if addr & 0x0100 != 0 {
            return 0x90
        }

        let port = addr as u8;
        if addr > 0xFF && port > 0xB8 {
            return 0x90
        }

        self.io_in[port as usize]
    }

    fn color_mode(&self) -> bool {
        self.read_io(0x60) >> 7 != 0
    }

    #[cfg(test)]
    pub fn set_wram(&mut self, wram: Vec<u8>) {
        for i in 0..wram.len() {
            self.wram[i] = wram[i];
        }
    }

    #[cfg(test)]
    pub fn set_io(&mut self, io: Vec<u8>) {
        for i in 0..io.len() {
            self.io_in[i] = io[i];
        }
    }

    #[cfg(test)]
    pub fn get_cpu(&mut self) -> &mut V30MZ {
        &mut self.cpu
    }

    #[cfg(test)]
    pub fn get_wram(&mut self) -> &mut [u8; 0xFFFF] {
        &mut self.wram
    }
}

#[cfg(test)]
mod test {
    use crate::assert_eq_hex;

    use super::SoC;

    #[test]
    fn test_io_open_bus() {
        let soc = SoC::new();
        assert_eq_hex!(soc.read_io(0x100), 0x90);
        assert_eq_hex!(soc.read_io(0x1B9), 0x90);
    }
}