use std::{cell::RefCell, rc::Rc};

use crate::cartridge::{Cartridge, Mapper};

use super::io_bus::{IOBus, IOBusConnection};

#[derive(PartialEq, Eq)]
pub enum Owner {
    NONE,
    CPU,
    DMA,
}

pub struct MemBus {
    pub owner: Owner,

    pub wram: [u8; 0x10000],

    pub cartridge: Cartridge,

    pub io_bus: Rc<RefCell<IOBus>>,
}

pub trait MemBusConnection {
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

impl MemBusConnection for MemBus {
    fn read_mem(&mut self, addr: u32) -> u8 {
        match addr {
            0x00000..=0x01FFF => self.wram[addr as usize],
            0x02000..=0x0FFFF => {
                if self.io_bus.borrow_mut().color_mode() {
                    self.wram[addr as usize]
                } else {
                    0x90
                }
            }
            addr => panic!("Not yet implemented! Addr: {:05X}", addr)
        }
    }

    fn write_mem(&mut self, addr: u32, byte: u8) {
        match addr {
            0x00000..=0x01FFF => self.wram[addr as usize] = byte,
            0x02000..=0x0FFFF => if self.io_bus.borrow_mut().color_mode() {self.wram[addr as usize] = byte}
            _ => todo!()
        }
    }
}

impl IOBusConnection for MemBus {
    fn read_io(&mut self, addr: u16) -> u8 {
        self.io_bus.borrow_mut().read_io(addr)
    }

    fn write_io(&mut self, addr: u16, byte: u8) {
        self.io_bus.borrow_mut().write_io(addr, byte);
    }
}

impl MemBus {
    pub fn new(io_bus: Rc<RefCell<IOBus>>, sram: Vec<u8>, rom: Vec<u8>, mapper: Mapper, rewrittable: bool) -> Self {
        let cartridge_io = Rc::clone(&io_bus);
        Self {owner: Owner::NONE, wram: [0; 0x10000], io_bus, cartridge: Cartridge::new(cartridge_io, mapper, sram, rom, rewrittable)}
    }

    #[cfg(test)]
    pub fn test_build(io_bus: Rc<RefCell<IOBus>>) -> Self {
        let cartridge_io = Rc::clone(&io_bus);
        Self {owner: Owner::NONE, wram: [0; 0x10000], io_bus, cartridge: Cartridge::test_build(cartridge_io)}
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use std::ops::{Index, IndexMut};

    #[cfg(test)]
    impl Index<usize> for MemBus {
        type Output = u8;

        fn index(&self, index: usize) -> &Self::Output {
            &self.wram[index]
        }
    }

    #[cfg(test)]
    impl IndexMut<usize> for MemBus {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            &mut self.wram[index]
        }
    }
}