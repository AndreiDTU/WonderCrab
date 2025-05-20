use crate::cpu::v30mz::V30MZ;

pub struct SoC {
    // COMPONENTS
    cpu: V30MZ,

    // MEMORY
    wram: [u8; 0xFFFF],

    // I/O
    io: [u8; 0xFF],
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
            if self.cpu.mem_request {
                let addr = self.cpu.get_pc_address();
                let byte = self.read_mem(addr);
                self.cpu.current_op.push(byte);
                self.cpu.execute();
                continue;
            }

            if self.cpu.io_request {
                let addr = self.cpu.get_io_address();
                let byte = self.read_io(addr);
                self.cpu.io_response.push(byte);
                self.cpu.execute();
                continue;
            }

            break;
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

    pub fn read_io(&self, addr: u16) -> u8 {
        if addr & 0x0100 != 0 {
            return 0x90
        }

        let addr = addr as u8;
        if addr > 0xB8 {
            return 0x90
        }

        self.io[addr as usize]
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
            self.io[i] = io[i];
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cpu_in_0xe5() {
        let mut soc = SoC::new();
        soc.set_wram(vec![0xE5, 0x00]);
        soc.set_io(vec![0xCD, 0xAB]);
        soc.tick();
        assert_eq!(soc.cpu.get_aw(), 0xABCD);
    }
}