use std::{cell::RefCell, rc::Rc};

use crate::cpu::v30mz::V30MZ;

pub struct SoC {
    // COMPONENTS
    cpu: V30MZ,

    // MEMORY
    wram: Rc<RefCell<[u8; 0x10000]>>,

    // I/O
    io: Rc<RefCell<[u8; 0x100]>>,
}

pub trait MemBus {
    fn read_mem(&mut self, addr: u32) -> u8;
    fn write_mem(&mut self, addr: u32, byte: u8);

    fn read_mem_16(&mut self, addr: u32) -> u16{
        let bytes = [self.read_mem(addr), self.read_mem(addr.wrapping_add(1))];
        u16::from_le_bytes(bytes)
    }

    fn write_mem_16(&mut self, addr: u32, src: u16) {
        let bytes = src.to_le_bytes();
        self.write_mem(addr, bytes[0]);
        self.write_mem(addr.wrapping_add(1), bytes[1]);
    }

    fn read_mem_32(&mut self, addr: u32) -> (u16, u16){
        let bytes1 = [self.read_mem(addr), self.read_mem(addr.wrapping_add(1))];
        let bytes2 = [self.read_mem(addr.wrapping_add(2)), self.read_mem(addr.wrapping_add(3))];
        let result1 = u16::from_le_bytes(bytes1);
        let result2 = u16::from_le_bytes(bytes2);
        (result1, result2)
    }
}

pub trait IOBus {
    fn read_io(&mut self, addr: u16) -> u8;
    fn write_io(&mut self, addr: u16, byte: u8);

    fn read_io_16(&mut self, addr: u16) -> (u8, u8) {
        (self.read_io(addr), self.read_io(addr.wrapping_add(1)))
    }

    fn write_io_16(&mut self, addr: u16, byte: u8) {
        self.write_io(addr, byte);
        self.write_io(addr.wrapping_add(1), byte);
    }
}

impl MemBus for SoC {
    fn read_mem(&mut self, addr: u32) -> u8 {
        match addr {
            0x00000..=0x03FFF => self.wram.borrow()[addr as usize],
            0x04000..=0x0FFFF => {
                if !self.color_mode() {
                    0x90
                } else {
                    self.wram.borrow()[addr as usize]
                }
            }
            addr => panic!("Not yet implemented! Addr: {:05X}", addr)
        }
    } 

    fn write_mem(&mut self, addr: u32, data: u8) {
        match addr {
            0x00000..=0x03FFF => self.wram.borrow_mut()[addr as usize] = data,
            0x04000..=0x0FFFF => if self.color_mode() {
                self.wram.borrow_mut()[addr as usize] = data;
            }
            _ => todo!()
        }
    }
}

impl IOBus for SoC {
    fn read_io(&mut self, addr: u16) -> u8 {
        if addr & 0x0100 != 0 {
            return 0x90
        }

        let port = addr as u8;
        if addr > 0xFF && port > 0xB8 {
            return 0x90
        }

        self.io.borrow()[port as usize]
    }
    
    fn write_io(&mut self, addr: u16, byte: u8) {
        self.io.borrow_mut()[addr as usize] = byte
    }
}

impl SoC {
    pub fn new() -> Self {
        let wram = Rc::new(RefCell::new([0; 0x10000]));
        let io = Rc::new(RefCell::new([0; 0x100]));
        let cpu = V30MZ::new(Rc::clone(&wram), Rc::clone(&io));

        Self {cpu, wram, io}
    }

    pub fn tick(&mut self) {
        self.cpu.tick();
    }

    fn color_mode(&mut self) -> bool {
        self.read_io(0x60) >> 7 != 0
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod test;