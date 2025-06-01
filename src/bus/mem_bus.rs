use std::{cell::RefCell, rc::Rc};

use crate::cartridge::Cartridge;

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

    pub cartridge: Rc<RefCell<Cartridge>>,

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
            0x00000..=0x03FFF => self.wram[addr as usize],
            0x04000..=0x0FFFF => {
                if self.io_bus.borrow_mut().color_mode() {
                    self.wram[addr as usize]
                } else {
                    0x90
                }
            }
            0x10000..=0x1FFFF => self.cartridge.borrow().read_sram(addr),
            0x20000..=0x2FFFF => self.cartridge.borrow().read_rom_0(addr),
            0x30000..=0x3FFFF => self.cartridge.borrow().read_rom_1(addr),
            0x40000..=0xFFFFF => self.cartridge.borrow().read_rom_ex(addr),
            addr => panic!("Address {:08X} out of range!", addr)
        }
    }

    fn write_mem(&mut self, addr: u32, byte: u8) {
        // if (0x29C0..=0x29CF).contains(&addr) {println!("[{:04X}] <- {:02X}", addr, byte)}
        // if addr == 0x01000 {println!("[{:04X}] <- {:02X}", addr, byte)}
        match addr {
            0x00000..=0x03FFF => {
                self.wram[addr as usize] = byte;
                // println!("{:05X} <- {:02X}", addr, byte);
            }
            0x04000..=0x0FFFF => if self.io_bus.borrow_mut().color_mode() {self.wram[addr as usize] = byte}
            0x10000..=0x1FFFF => self.cartridge.borrow_mut().write_sram(addr, byte),
            0x20000..=0xFFFFF => {
                // println!("Ignoring attempt to write to ROM {:05X} <- {:02X}", addr, byte),
            }
            addr => panic!("Address {:08X} out of range! Attempting to write {:02X}", addr, byte),
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
    pub fn new(io_bus: Rc<RefCell<IOBus>>, cartridge: Rc<RefCell<Cartridge>>) -> Self {
        Self {owner: Owner::NONE, wram: [0; 0x10000], io_bus, cartridge}
    }

    pub fn test_build(io_bus: Rc<RefCell<IOBus>>, cartridge: Rc<RefCell<Cartridge>>) -> Self {
        Self {owner: Owner::NONE, wram: [0; 0x10000], io_bus, cartridge}
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