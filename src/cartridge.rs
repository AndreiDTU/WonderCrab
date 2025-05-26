use std::{cell::RefCell, rc::Rc};

use crate::bus::io_bus::{IOBus, IOBusConnection};

pub mod bank_access;

pub enum Mapper {
    B_2001,
    B_2003,
}

pub struct Cartridge {
    sram: Vec<u8>,
    rom: Vec<u8>,

    io_bus: Rc<RefCell<IOBus>>,

    mapper: Mapper,

    RAM_BANK_L: u8,
    RAM_BANK_H: u8,

    ROM_BANK_0_L: u8,
    ROM_BANK_0_H: u8,
    ROM_BANK_1_L: u8,
    ROM_BANK_1_H: u8,

    rewrittable: bool,
}

impl IOBusConnection for Cartridge {
    fn read_io(&mut self, addr: u16) -> u8 {
        self.io_bus.borrow_mut().read_io(addr)
    }

    fn write_io(&mut self, addr: u16, byte: u8) {
        self.io_bus.borrow_mut().write_io(addr, byte);
    }
}

impl Cartridge {
    pub fn new(io_bus: Rc<RefCell<IOBus>>, mapper: Mapper, sram: Vec<u8>, rom: Vec<u8>, rewrittable: bool) -> Self {
        Self {
            sram, rom, io_bus, mapper,
            RAM_BANK_L: 0xFF, RAM_BANK_H: 0xFF,
            ROM_BANK_0_L: 0xFF, ROM_BANK_0_H: 0xFF,
            ROM_BANK_1_L: 0xFF, ROM_BANK_1_H: 0xFF,
            rewrittable,
        }
    }

    pub fn read_sram(&self, addr: u32) -> u8 {
        let hi = match self.mapper {
            Mapper::B_2001 => self.RAM_BANK_L as u32,
            Mapper::B_2003 => u16::from_le_bytes([self.RAM_BANK_L, self.RAM_BANK_H]) as u32,
        };
        let lo = addr & 0xFFFF;

        let offset = (hi << 16) | lo;

        self.sram[offset as usize]
    }

    pub fn write_sram(&mut self, addr: u32, byte: u8) {
        if self.rewrittable {
            let hi = match self.mapper {
                Mapper::B_2001 => self.RAM_BANK_L as u32,
                Mapper::B_2003 => u16::from_le_bytes([self.RAM_BANK_L, self.RAM_BANK_H]) as u32,
            };
            let lo = addr & 0xFFFF;

            let offset = (hi << 16) | lo;

            self.sram[offset as usize] = byte;
        }
    }

    pub fn read_rom_0(&self, addr: u32) -> u8 {
        let hi = match self.mapper {
            Mapper::B_2001 => self.ROM_BANK_0_L as u32,
            Mapper::B_2003 => u16::from_le_bytes([self.ROM_BANK_0_L, self.ROM_BANK_0_H]) as u32,
        };
        let lo = addr & 0xFFFF;

        let offset = (hi << 16) | lo;

        self.rom[offset as usize]
    }

    pub fn read_rom_1(&self, addr: u32) -> u8 {
        let hi = match self.mapper {
            Mapper::B_2001 => self.ROM_BANK_1_L as u32,
            Mapper::B_2003 => u16::from_le_bytes([self.ROM_BANK_1_L, self.ROM_BANK_1_H]) as u32,
        };
        let lo = addr & 0xFFFF;

        let offset = (hi << 16) | lo;

        self.rom[offset as usize]
    }

    #[cfg(test)]
    pub fn test_build(io_bus: Rc<RefCell<IOBus>>) -> Self {
        Self::new(io_bus, Mapper::B_2001, Vec::new(), Vec::new(), true)
    }
}