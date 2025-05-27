use std::{cell::RefCell, rc::Rc};

use crate::{cartridge::Cartridge, display::PaletteFormat};

pub struct IOBus {
    ports: [u8; 0x100],

    cartridge: Rc<RefCell<Cartridge>>,
}

pub trait IOBusConnection {
    fn read_io(&mut self, addr: u16) -> u8;
    fn write_io(&mut self, addr: u16, byte: u8);

    fn read_io_16(&mut self, addr: u16) -> (u8, u8) {
        (self.read_io(addr), self.read_io(addr.wrapping_add(1)))
    }

    fn write_io_16(&mut self, addr: u16, word: u16) {
        let bytes = word.to_le_bytes();
        self.write_io(addr, bytes[0]);
        self.write_io(addr.wrapping_add(1), bytes[1]);
    }
}

impl IOBusConnection for IOBus {
    fn read_io(&mut self, addr: u16) -> u8 {
        let Some(port) = Self::check_open_bus(addr) else {return Self::open_bus()};

        match port {
            // Lowest bit of GDMA_SOURCE_L is always clear
            0x41 => self.ports[0x41] & 0xFE,

            // Bits 4-15 of GDMA_SOURCE_H are undefined
            0x42 => 0,
            0x43 => self.ports[0x43] & 0x0F,

            // Lowest bit of GDMA_DESTINATION is always clear
            0x45 => self.ports[0x45] & 0xFE,

            // Lowest bit of GDMA_COUNTER is always clear
            0x47 => self.ports[0x47] & 0xFE,

            // Lowest bits of GDMA_CTRL are undefined on read
            // GDMA_CTRL clears on read
            0x48 => {
                let output = self.ports[0x48] & 0xC0;
                self.ports[0x48] = 0;
                output
            }

            // INT_CAUSE_CLEAR is write-only
            0xB6 => 0,

            // INT_NMI_CTRL clears most of its bits when read
            0xB7 => {
                self.ports[0xB7] &= 0x10;
                self.ports[0xB7]
            }

            // CARTRIDGE PORTS
            0xC0 => self.cartridge.borrow().read_linear_addr_off(),
            0xC1 => self.cartridge.borrow().read_ram_bank(),
            0xC2 => self.cartridge.borrow().read_rom_bank_0(),
            0xC3 => self.cartridge.borrow().read_rom_bank_1(),
            0xCF => self.cartridge.borrow().read_linear_addr_off_shadow(),
            0xD0 => self.cartridge.borrow().read_ram_bank_l(),
            0xD1 => self.cartridge.borrow().read_ram_bank_h(),
            0xD2 => self.cartridge.borrow().read_rom_bank_0_l(),
            0xD3 => self.cartridge.borrow().read_rom_bank_0_h(),
            0xD4 => self.cartridge.borrow().read_rom_bank_1_l(),
            0xD5 => self.cartridge.borrow().read_rom_bank_1_h(),

            // Default no side-effects
            _ => self.ports[port as usize]
        }
    }

    fn write_io(&mut self, addr: u16, byte: u8) {
        let Some(port) = Self::check_open_bus(addr) else {return};

        match port {
            // Lowest bit of GDMA_SOURCE_L is always clear
            0x41 => self.ports[0x41] = byte & 0xFE,

            // Lowest bit of GDMA_DESTINATION is always clear
            0x45 => self.ports[0x45] = byte & 0xFE,

            // Lowest bit of GDMA_COUNTER is always clear
            0x47 => self.ports[0x47] = byte & 0xFE,

            // INT_CAUSE is read-only
            0xB4 => {}

            // INT_CAUSE_CLEAR clears bits of INT_CAUSE when written to
            0xB6 => {
                self.ports[0xB6] = byte;
                self.ports[0xB4] &= !self.ports[0xB6]
            }

            // CARTRIDGE PORTS
            0xC0 => self.cartridge.borrow_mut().write_linear_addr_off(byte),
            0xC1 => self.cartridge.borrow_mut().write_ram_bank(byte),
            0xC2 => self.cartridge.borrow_mut().write_rom_bank_0(byte),
            0xC3 => self.cartridge.borrow_mut().write_rom_bank_1(byte),
            0xCF => {}
            0xD0 => self.cartridge.borrow_mut().write_ram_bank_l(byte),
            0xD1 => self.cartridge.borrow_mut().write_ram_bank_h(byte),
            0xD2 => self.cartridge.borrow_mut().write_rom_bank_0_l(byte),
            0xD3 => self.cartridge.borrow_mut().write_rom_bank_0_h(byte),
            0xD4 => self.cartridge.borrow_mut().write_rom_bank_1_l(byte),
            0xD5 => self.cartridge.borrow_mut().write_rom_bank_1_h(byte),

            // Default no side-effects
            _ => self.ports[port as usize] = byte
        }
    }
}

impl IOBus {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> Self {
        Self {ports: [0; 0x100], cartridge}
    }

    pub fn color_mode(&mut self) -> bool {
        self.read_io(0x60) >> 7 != 0
    }

    pub fn pallete_format(&mut self) -> PaletteFormat {
        if !self.color_mode() {
            PaletteFormat::PLANAR_2BPP
        } else {
            match self.read_io(0x60) {
                0b100 | 0b101 => PaletteFormat::PLANAR_2BPP,
                0b110 => PaletteFormat::PLANAR_4BPP,
                0b111 => PaletteFormat::PACKED_4BPP,
                _ => unreachable!()
            }
        }
    }

    pub fn color_setup(&mut self) {
        self.ports[0x60] = 0x80;
    }

    pub fn open_bus() -> u8 {
        0x90
    }

    fn check_open_bus(addr: u16) -> Option<u8> {
        if addr & 0x0100 != 0 {
            return None;
        }

        let port = addr as u8;
        if addr > 0xFF && port > 0xB8 {
            return None;
        }

        return Some(port);
    }
}