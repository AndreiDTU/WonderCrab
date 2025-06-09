use std::{cell::RefCell, rc::Rc};

use crate::cartridge::Cartridge;

use super::io_bus::IOBus;

/// Used for moments when the memory bus is owned exclusively by one component or another
#[derive(PartialEq, Eq)]
pub enum Owner {
    /// The default state of the bus
    NONE,
    /// Used when the CPU calls the BUSLOCK instruction
    CPU,
    /// Used when the GDMA is operating
    DMA,
}

/// The WonderSwan's shared memory bus
pub struct MemBus {
    /// The bus's current owner
    pub owner: Owner,

    /// WonderSwan's internal work RAM, only a quarter of it is accessible on monochrome models
    pub wram: [u8; 0x10000],

    /// A reference to the cartridge, shared with the I/O bus
    pub cartridge: Rc<RefCell<Cartridge>>,

    /// A reference to the I/O bus, only used to check if color mode is enabled
    pub io_bus: Rc<RefCell<IOBus>>,
}

/// Trait shared by objects containing references to the shared memory bus
/// 
/// The memory map of the WonderSwan's 20-bit addressing space is as follows:
/// 
/// | Address           | Memory            |
/// |-------------------|-------------------|
/// | 0x00000 - 0x03FFF | WRAM              |
/// | 0x04000 - 0x0FFFF | WRAM (color only) |
/// | 0x10000 - 0x1FFFF | SRAM              |
/// | 0x20000 - 0x2FFFF | ROM bank 0        |
/// | 0x30000 - 0x3FFFF | ROM bank 1        |
/// | 0x40000 - 0xFFFFF | ROM EX range      |
pub trait MemBusConnection {
    /// Returns the byte at the address
    /// 
    /// # Panics
    /// This function will panic when the address is greater than 0xFFFFF
    fn read_mem(&mut self, addr: u32) -> u8;
    /// Writes the byte to the address
    /// 
    /// # Panics
    /// This function will panic when the address is greater than 0xFFFFF
    fn write_mem(&mut self, addr: u32, byte: u8);


    /// Returns the word read from the address given and the following address, interpreted in little-endian form
    /// 
    /// # Panics
    /// This function will panic when the address is greater than 0xFFFFE
    fn read_mem_16(&mut self, addr: u32) -> u16 {
        let bytes = [self.read_mem(addr), self.read_mem(addr.wrapping_add(1))];
        u16::from_le_bytes(bytes)
    }

    /// Writes the word to the address given and the following address, interpreted in little-endian form
    /// 
    /// # Panics
    /// This function will panic when the address is greater than 0xFFFFE
    fn write_mem_16(&mut self, addr: u32, src: u16) {
        let bytes = src.to_le_bytes();
        self.write_mem(addr, bytes[0]);
        self.write_mem(addr.wrapping_add(1), bytes[1]);
    }

    /// Reads four bytes from the provided address and the following three, returns two 16-bit values, which are the
    /// result of interpreting each pair of bytes as a word in little-endian form
    /// 
    /// # Panics
    /// This function will panic when the address is greater than 0xFFFFC
    fn read_mem_32(&mut self, addr: u32) -> (u16, u16) {
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

impl MemBus {
    /// Creates a new I/O bus, requires references to the I/O bus and cartridge
    pub fn new(io_bus: Rc<RefCell<IOBus>>, cartridge: Rc<RefCell<Cartridge>>) -> Self {
        Self {owner: Owner::NONE, wram: [0; 0x10000], io_bus, cartridge}
    }

    /// A test build used during tests or if the user does not provide a ROM
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