use std::{cell::RefCell, rc::Rc};

use eeprom::EEPROM;

use crate::{bus::io_bus::keypad::{Keypad, Keys}, cartridge::Cartridge, display::PaletteFormat};

/// IEEPROM and cartridge EEPROM
/// 
/// The console's EEPROM was mostly used to store certain information about the owner,
/// with only a small section being rewrittable by the games themselves.
/// 
/// Cartridges with EEPROM save files typically used them for small amounts of data such as high score records.
mod eeprom;
/// Module used for inputs
/// 
/// The keypad represents all of the system's built-in buttons.
pub(crate) mod keypad;

/// The WonderSwan's shared I/O bus
pub struct IOBus {
    /// This is an array containing the byte at each port
    ports: [u8; 0x100],

    /// A reference to the cartridge, shared with the memory bus
    pub(crate) cartridge: Rc<RefCell<Cartridge>>,
    /// The cartridge's EEPROM, is none in case the cartridge instead contains SRAM
    pub(crate) eeprom: Option<EEPROM>,
    /// The system's internal EEPROM
    pub(crate) ieeprom: EEPROM,

    /// The console's built-in keys
    keypad: Keypad,
}

/// Trait shared by objects which are connected to the I/O bus
///
/// This trait is intended to be implemented on any struct containing a reference to the I/O bus.
/// 
/// However, structs which can communicate exclusively via the I/O bus are instead expected
/// to be contained as fields of the I/O bus.
pub trait IOBusConnection {
    /// Returns the byte at the port indicated by the address
    /// 
    /// Reading certain ports can have side effects.
    /// 
    /// Some ports do not exist and return undefined values, some ports also have undefined bits.
    /// 
    /// The addressing space for ports is also larger than the amount of existing ports so some addresses are mirrored and others are invalid.
    fn read_io(&mut self, addr: u16) -> u8;
    /// Writes a byte to the address
    /// 
    /// Some ports or bits of certain ports cannot be written to.
    /// 
    /// Writing to some ports can have side effects.
    /// 
    /// The addressing space for ports is also larger than the amount of existing ports so some addresses are mirrored and others are invalid.
    fn write_io(&mut self, addr: u16, byte: u8);

    /// Reads the byte at the address provided and the following address, returns a tuple containing both bytes
    fn read_io_16(&mut self, addr: u16) -> (u8, u8) {
        (self.read_io(addr), self.read_io(addr.wrapping_add(1)))
    }
    /// Writes the word in little-endian form to the address provided and the following one
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
            0x40 => self.ports[0x40] & 0xFE,

            // Bits 4-15 of GDMA_SOURCE_H are undefined
            0x42 => self.ports[0x42] & 0x0F,
            0x43 => 0,

            // Lowest bit of GDMA_DESTINATION is always clear
            0x44 => self.ports[0x44] & 0xFE,

            // Lowest bit of GDMA_COUNTER is always clear
            0x46 => self.ports[0x46] & 0xFE,

            // Lowest bits of GDMA_CTRL are undefined on read
            // GDMA_CTRL clears on read
            0x48 => {
                let output = self.ports[0x48] & 0xC0;
                self.ports[0x48] = 0;
                output
            }

            0x4C => self.ports[0x4C] & 0x0F,
            0x4D => 0,

            0x50 => self.ports[0x50] & 0x0F,
            0x51 => 0,

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
                self.ports[0xB5] | self.keypad.read_keys()
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

            // EEPROM ports
            0xC4..=0xC7 => if self.eeprom.is_some() {self.ports[port as usize]} else {Self::open_bus()}

            0xC8 => if self.eeprom.is_some() {2} else {Self::open_bus()},
            0xC9 => Self::open_bus(),

            0xBA | 0xBB => 0,

            0xBE => 0x83,
            0xBF => 0,

            // Default no side-effects
            _ => self.ports[port as usize]
        }
    }

    fn write_io(&mut self, addr: u16, byte: u8) {
        let Some(port) = Self::check_open_bus(addr) else {return};
        // println!("{:02X} <- {:02X}", port, byte);
        // if (0xC4..=0xC9).contains(&addr) {println!("Cart EEPROM operation at {:02X}", port)}
        // if (0xBA..=0xBF).contains(&addr) {println!("IEEPROM operation at {:02X}", port)}

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
            0x40 => self.ports[0x40] = byte & 0xFE,

            // Bits 4-15 of GDMA_SOURCE_H are undefined
            0x42 => self.ports[0x42] = byte & 0x0F,
            0x43 => {},

            // Lowest bit of GDMA_DESTINATION is always clear
            0x44 => self.ports[0x44] = byte & 0xFE,

            // Lowest bit of GDMA_COUNTER is always clear
            0x46 => self.ports[0x46] = byte & 0xFE,

            0x4C => self.ports[0x4C] = byte & 0x0F,
            0x4D => {},

            0x50 => self.ports[0x50] = byte & 0x0F,
            0x51 => {},

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
                let old_keys = self.ports[0xB5] & 0x0F;
                self.keypad.poll((byte & 0x70) >> 4);
                self.ports[0xB5] = (byte & 0x70) | self.keypad.read_keys();
                if (self.ports[0xB5] & 0x0F) != old_keys {
                    // println!("Keys pressed!");
                    self.ports[0xB4] |= (1 << 1) & self.ports[0xB2];
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

            // EEPROM ports
            0xC4..=0xC7 => if self.eeprom.is_some() {
                // println!("[{:02X}] <- {:02X}", port, byte);
                self.ports[port as usize] = byte;
            }

            0xC8 => if let Some(eeprom) = &mut self.eeprom {
                self.ports[0xC8] = byte & 0xF0;
                let operation = byte >> 4;
                // println!("Cart EEPROM operation: {:04b}", operation);
                match operation {
                    0b0001 => {
                        eeprom.write_comm(u16::from_le_bytes([self.ports[0xC6], self.ports[0xC7]]));
                        [self.ports[0xC4], self.ports[0xC5]] = eeprom.read_data().to_le_bytes();
                        // println!("Read data from EEPROM: {:04X}", u16::from_le_bytes([self.ports[0xC4], self.ports[0xC5]]))
                    }
                    0b0010 => {
                        let data = u16::from_le_bytes([self.ports[0xC4], self.ports[0xC5]]);
                        let comm = u16::from_le_bytes([self.ports[0xC6], self.ports[0xC7]]);
                        eeprom.write_data(data);
                        eeprom.write_comm(comm);
                        // println!("data: {:04X}, comm: {:04X}", data, comm);
                    }
                    0b0100 => eeprom.write_comm(u16::from_le_bytes([self.ports[0xC6], self.ports[0xC7]])),
                    _ => {}
                }
            }
            0xC9 => {},

            0xBE => {
                self.ports[0xBE] = byte & 0xF0;
                let operation = byte >> 4;
                let comm = u16::from_le_bytes([self.ports[0xBC], self.ports[0xBD]]);
                if operation != 0b0001 {
                    let address_bits = if self.color_mode() {10} else {6};
                    if (comm & ((1 << address_bits) - 1)) * 2 >= 0x60 {
                        return;
                    }
                }
                match operation {
                    0b0001 => {
                        self.ieeprom.write_comm(comm);
                        [self.ports[0xBA], self.ports[0xBB]] = self.ieeprom.read_data().to_le_bytes();
                    }
                    0b0010 => {
                        let data = u16::from_le_bytes([self.ports[0xBA], self.ports[0xBB]]);
                        self.ieeprom.write_data(data);
                        self.ieeprom.write_comm(comm);
                    }
                    0b0100 => self.ieeprom.write_comm(comm),
                    _ => {}
                }
            },
            0xBF => {},

            // Default no side-effects
            _ => self.ports[port as usize] = byte
        }
    }
}

impl IOBus {
    /// Returns a new I/O bus object
    /// 
    /// Requires the IEEPROM, an optional cartridge EEPROM, a boolean indicating whether to run in color mode, info about the ROM and a shared reference to the cartridge.
    pub fn new(cartridge: Rc<RefCell<Cartridge>>, ieeprom: Vec<u8>, eeprom: Option<Vec<u8>>, color: bool, rom_info: u8) -> Self {
        let ieeprom = if ieeprom.is_empty() {
            if color {
                EEPROM::new(vec![0; 0x800], 10)
            } else {
                EEPROM::new(vec![0; 128], 6)
            }
        } else {
            EEPROM::new(ieeprom, if color {10} else {6})
        };
        
        let eeprom = if let Some(contents) = eeprom {
            let address_bits = match contents.len() {
                0x400 => 6,
                0x2000 | 0x4000 => 10,
                _ => panic!("Unsupported EEPROM size {:X}", contents.len())
            };
            Some(EEPROM::new(contents, address_bits))
        } else {None};
        
        let mut bus = Self {ports: [0; 0x100], cartridge, keypad: Keypad::new(), eeprom, ieeprom};
        if color {bus.color_setup()};
        bus.ports[0xA0] |= rom_info;
        bus
    }

    /// Returns whether or not the console is in color mode as indicated by port 0x60
    pub fn color_mode(&mut self) -> bool {
        self.ports[0x60] >> 7 != 0
    }

    /// Returns the format of the palette data as indicated by port 0x60
    pub fn palette_format(&mut self) -> PaletteFormat {
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

    /// Sets the values of ports 0x60 and 0xA0 to what would be expected in a WonderSwan Color model with color mode enabled
    pub fn color_setup(&mut self) {
        self.ports[0x60] = 0x80;
        self.ports[0xA0] = 0x86;
    }

    /// Returns 0x90 to simulate the open bus behaviour of the monochrome WonderSwan.
    /// 
    /// It is its own separate function in order to make it easier to potentially simulate a more accurate version of
    /// WonderSwan Color and WonderCrystal open bus behaviour in the future.
    pub fn open_bus() -> u8 {
        0x90
    }

    /// Sets the state of a key to be either pressed or unpressed
    pub fn set_key(&mut self, key: Keys, pressed: bool) {
        self.keypad.set_key(key, pressed);
        let old_keys = self.ports[0xB5];
        self.ports[0xB5] |= self.keypad.read_keys();
        if !old_keys & self.ports[0xB5] != 0 {
            self.ports[0xB4] |= (1 << 1) & self.ports[0xB2];
        }
    }

    // Display functions

    /// Called by the display controller to announce its current scanline
    /// 
    /// # Interrupt
    /// Can potentially trigger the DISPLINE interrupt if enabled and the scanline matches port 0x03
    pub(crate) fn set_lcd_line(&mut self, line: u8) {
        self.ports[0x02] = line;
        if self.ports[0x02] == self.ports[0x03] {
            self.ports[0xB4] |= (1 << 4) & self.ports[0xB2];
        }
    }

    /// Called by the display controller to announce that it has finished rendering a frame
    /// 
    /// # Interrupt
    /// Can trigger the VBLANK and VBLANK_COUNTER interrupts if enabled and their conditions are met
    pub (crate) fn vblank(&mut self) {
        self.ports[0xB4] |= (1 << 6) & self.ports[0xB2];
        if self.ports[0xA2] & 4 != 0 {
            let counter = u16::from_le_bytes([self.ports[0xAA], self.ports[0xAB]]);
            if counter == 1 {
                self.ports[0xB4] |= (1 << 5) & self.ports[0xB2];
                if self.ports[0xA2] & 8 != 0 {
                    self.ports[0xAA] = self.ports[0xA6];
                    self.ports[0xAB] = self.ports[0xA7];
                } else {
                    self.ports[0xAA] = 0;
                    self.ports[0xAB] = 0;
                }
            } else {
                let counter = counter - 1;
                [self.ports[0xAA], self.ports[0xAB]] = counter.to_le_bytes();
            }
        }
    }

    /// Called by the display controller to announce it has finished rendering a scanline
    /// 
    /// # Interrupt
    /// Can trigger the HBLANK_COUNTER interrupt if enabled and is condition is met
    pub (crate) fn hblank(&mut self) {
        if self.ports[0xA2] & 1 != 0 {
            let counter = u16::from_le_bytes([self.ports[0xA8], self.ports[0xA9]]);
            if counter == 1 {
                self.ports[0xB4] |= (1 << 7) & self.ports[0xB2];
                if self.ports[0xA2] & 2 != 0 {
                    self.ports[0xA8] = self.ports[0xA4];
                    self.ports[0xA9] = self.ports[0xA5];
                } else {
                    self.ports[0xA8] = 0;
                    self.ports[0xA9] = 0;
                }
            } else {
                let counter = counter - 1;
                [self.ports[0xA8], self.ports[0xA9]] = counter.to_le_bytes();
            }
        }
    }

    // Sound functions

    /// Called by the sound chip to announce the state of the LSFR
    /// 
    /// This port can potentially be read by the CPU as a form of pseudo-RNG
    pub(crate) fn set_lsfr(&mut self, lsfr: u16) {
        let bytes = lsfr.to_le_bytes();
        self.write_io(0x92, bytes[0]);
        self.write_io(0x93, bytes[1]); 
    }

    /// Transforms the 16-bit address received by the bus into an 8-bit index
    /// 
    /// # Returns
    /// - None, if the address is invalid
    /// - Some(port), the port mirrored into the valid addressing space
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

    #[allow(dead_code)]
    #[doc(hidden)]
    pub(crate) fn debug_eeprom(&self) {
        println!("IEEPROM {:#?}", self.ieeprom.contents);
        if let Some(eeprom) = &self.eeprom {
            println!("CART EEPROM: {:#?}", eeprom.contents);
        }
    }
}