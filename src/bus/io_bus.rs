use std::{cell::RefCell, rc::Rc};

use crate::{cartridge::Cartridge, display::PaletteFormat, keypad::Keypad};

pub struct IOBus {
    ports: [u8; 0x100],

    cartridge: Rc<RefCell<Cartridge>>,
    pub(in crate) keypad: Rc<RefCell<Keypad>>,
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
        // println!("Reading from {:02X}", port);

        match port {
            // SCR_LUT ports have undefined bits
            0x28 | 0x2A | 0x2C | 0x2E | 0x38 | 0x3A | 0x3C | 0x3E => self.ports[addr as usize] & 0x70,
            0x20..=0x3F => self.ports[addr as usize] & 0x77,

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

            // SYSTEM_CTRL_2 is color only
            0x60 => {
                if self.color_mode() {
                    self.ports[0x60]
                } else {
                    Self::open_bus()
                }
            }

            // VBLANK is always enabled in INT_ENABLE
            0xB2 => self.ports[0xB2] | (1 << 6),

            // SERIAL_STATUS
            0xB3 => 0x84,

            // Reading INT_CAUSE clears edge interrupts
            0xB4 => {
                let cause = self.ports[0xB4];
                self.ports[0xB4] &= !0b1111_0010;
                cause
            }

            // Reading from KEY_SCAN queries the keypad
            0xB5 => {
                self.ports[0xB5] | self.keypad.borrow().read_keys()
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
        // println!("{:02X} <- {:02X}", port, byte);

        match port {
            // DISPLAY_CTRL
            0x00 | 0x01 => {
                self.ports[port as usize] = byte;
            }
            // LCD_LINE is read-only
            0x02 => {}

            // SCR_LUT ports have undefined bits
            0x20..=0x3E => self.ports[addr as usize] = byte & 0x77,

            // Lowest bit of GDMA_SOURCE_L is always clear
            0x41 => self.ports[0x41] = byte & 0xFE,

            // Lowest bit of GDMA_DESTINATION is always clear
            0x45 => self.ports[0x45] = byte & 0xFE,

            // Lowest bit of GDMA_COUNTER is always clear
            0x47 => self.ports[0x47] = byte & 0xFE,

            // Writing to HBLANK and VBLANK timers also sets the counters
            0xA4 => {
                self.ports[0xA4] = byte;
                self.ports[0xA8] = byte;
            }
            0xA5 => {
                self.ports[0xA5] = byte;
                self.ports[0xA9] = byte;
            }
            0xA6 => {
                self.ports[0xA6] = byte;
                self.ports[0xAA] = byte;
            }
            0xA7 => {
                self.ports[0xA7] = byte;
                self.ports[0xAB] = byte;
            }

            // Counters are read-only
            0xA8 | 0xA9 | 0xAA | 0xAB => {}

            // VBLANK is always enabled in INT_ENABLE
            0xB2 => self.ports[0xB2] = byte | (1 << 6),

            // SERIAL_STATUS is read-only
            0xB3 => {}

            // INT_CAUSE is read-only
            0xB4 => {}

            // Writing to KEY_SCAN polls the keypad and potentially interrupts
            0xB5 => {
                let old_keys = self.keypad.borrow().read_keys();
                self.ports[0xB5] = (self.ports[0xB5] & 0x0F) | (byte & 0x70);
                self.keypad.borrow_mut().poll((byte & 0x70) >> 4);
                if self.keypad.borrow().read_keys() & (!old_keys) != 0 {
                    // println!("Keys pressed!");
                    self.ports[0xB4] |= 0x02 & self.ports[0xB2];
                }
            }

            // INT_CAUSE_CLEAR clears bits of INT_CAUSE when written to
            0xB6 => {
                self.ports[0xB6] = byte;
                self.ports[0xB4] &= !byte;
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
    pub fn new(cartridge: Rc<RefCell<Cartridge>>, keypad: Rc<RefCell<Keypad>>) -> Self {
        Self {ports: [0; 0x100], cartridge, keypad}
    }

    pub fn color_mode(&mut self) -> bool {
        self.ports[0x60] >> 7 != 0
    }

    pub fn pallete_format(&mut self) -> PaletteFormat {
        if !self.color_mode() {
            PaletteFormat::PLANAR_2BPP
        } else {
            match (self.read_io(0x60) >> 5) & 0b111 {
                0b100 | 0b101 => PaletteFormat::PLANAR_2BPP,
                0b110 => PaletteFormat::PLANAR_4BPP,
                0b111 => PaletteFormat::PACKED_4BPP,
                _ => unreachable!()
            }
        }
    }

    pub fn color_setup(&mut self) {
        self.ports[0x60] = 0x80;
        self.ports[0xA0] = 0x02;
    }

    pub fn open_bus() -> u8 {
        0x90
    }

    // Display functions
    pub(crate) fn set_lcd_line(&mut self, line: u8) {
        self.ports[0x02] = line;
        if self.ports[0x02] == self.ports[0x03] {
            self.ports[0xB4] |= (1 << 4) & self.ports[0xB2];
        }
    }

    pub (crate) fn vblank(&mut self) {
        self.ports[0xB4] |= (1 << 6) & self.ports[0xB2];
        if self.ports[0xA3] & 4 != 0 {
            let counter = u16::from_le_bytes([self.ports[0xAA], self.ports[0xAB]]);
            if counter == 0 {
                self.ports[0xB4] |= (1 << 5) & self.ports[0xB2];
                if self.ports[0xA3] & 8 != 0 {
                    self.ports[0xAA] = self.ports[0xA6];
                    self.ports[0xAB] = self.ports[0xA7];
                }
            } else {
                let counter = counter - 1;
                [self.ports[0xAA], self.ports[0xAB]] = counter.to_le_bytes();
            }
        }
    }

    pub (crate) fn hblank(&mut self) {
        if self.ports[0xA3] & 1 != 0 {
            let counter = u16::from_le_bytes([self.ports[0xA8], self.ports[0xA9]]);
            if counter == 0 {
                self.ports[0xB4] |= (1 << 7) & self.ports[0xB2];
                if self.ports[0xA3] & 2 != 0 {
                    self.ports[0xA8] = self.ports[0xA4];
                    self.ports[0xA9] = self.ports[0xA5];
                }
            } else {
                let counter = counter - 1;
                [self.ports[0xA8], self.ports[0xA9]] = counter.to_le_bytes();
            }
        }
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