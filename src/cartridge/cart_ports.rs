use crate::bus::io_bus::IOBus;

use super::{Cartridge, Mapper};

impl Cartridge {
    pub fn read_ram_bank(&self) -> u8 {
        self.RAM_BANK_L
    }

    pub fn read_ram_bank_l(&self) -> u8 {
        match self.mapper {
            Mapper::B_2001 => IOBus::open_bus(),
            Mapper::B_2003 => self.RAM_BANK_L,
        }
    }

    pub fn read_ram_bank_h(&self) -> u8 {
        match self.mapper {
            Mapper::B_2001 => IOBus::open_bus(),
            Mapper::B_2003 => self.RAM_BANK_H,
        }
    }

    pub fn read_rom_bank_0(&self) -> u8 {
        self.ROM_BANK_0_L
    }

    pub fn read_rom_bank_0_l(&self) -> u8 {
        match self.mapper {
            Mapper::B_2001 => IOBus::open_bus(),
            Mapper::B_2003 => self.ROM_BANK_0_L,
        }
    }

    pub fn read_rom_bank_0_h(&self) -> u8 {
        match self.mapper {
            Mapper::B_2001 => IOBus::open_bus(),
            Mapper::B_2003 => self.ROM_BANK_0_H,
        }
    }

    pub fn read_rom_bank_1(&self) -> u8 {
        self.ROM_BANK_1_L
    }

    pub fn read_rom_bank_1_l(&self) -> u8 {
        match self.mapper {
            Mapper::B_2001 => IOBus::open_bus(),
            Mapper::B_2003 => self.ROM_BANK_1_L,
        }
    }

    pub fn read_rom_bank_1_h(&self) -> u8 {
        match self.mapper {
            Mapper::B_2001 => IOBus::open_bus(),
            Mapper::B_2003 => self.ROM_BANK_1_H,
        }
    }

    pub fn read_linear_addr_off(&self) -> u8 {
        self.LINEAR_ADDR_OFF
    }

    pub fn read_linear_addr_off_shadow(&self) -> u8 {
        match self.mapper {
            Mapper::B_2001 => IOBus::open_bus(),
            Mapper::B_2003 => self.LINEAR_ADDR_OFF,
        }
    }

    pub fn write_ram_bank(&mut self, byte: u8) {
        self.RAM_BANK_L = byte;
    }

    pub fn write_ram_bank_l(&mut self, byte: u8) {
        match self.mapper {
            Mapper::B_2001 => {}
            Mapper::B_2003 => self.RAM_BANK_L = byte,
        }
    }

    pub fn write_ram_bank_h(&mut self, byte: u8) {
        match self.mapper {
            Mapper::B_2001 => {}
            Mapper::B_2003 => self.RAM_BANK_H = byte,
        }
    }

    pub fn write_rom_bank_0(&mut self, byte: u8) {
        self.ROM_BANK_0_L = byte;
    }

    pub fn write_rom_bank_0_l(&mut self, byte: u8) {
        match self.mapper {
            Mapper::B_2001 => {}
            Mapper::B_2003 => self.ROM_BANK_0_L = byte,
        }
    }

    pub fn write_rom_bank_0_h(&mut self, byte: u8) {
        match self.mapper {
            Mapper::B_2001 => {}
            Mapper::B_2003 => self.ROM_BANK_0_H = byte,
        }
    }

    pub fn write_rom_bank_1(&mut self, byte: u8) {
        self.ROM_BANK_1_L = byte;
    }

    pub fn write_rom_bank_1_l(&mut self, byte: u8) {
        match self.mapper {
            Mapper::B_2001 => {}
            Mapper::B_2003 => self.ROM_BANK_1_L = byte,
        }
    }

    pub fn write_rom_bank_1_h(&mut self, byte: u8) {
        match self.mapper {
            Mapper::B_2001 => {}
            Mapper::B_2003 => self.ROM_BANK_1_H = byte,
        }
    }

    pub fn write_linear_addr_off(&mut self, byte: u8) {
        self.LINEAR_ADDR_OFF = byte;
    }
}