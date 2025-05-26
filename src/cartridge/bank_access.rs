use super::{Cartridge, Mapper};

impl Cartridge {
    pub fn read_ram_bank(&self) -> u8 {
        self.RAM_BANK_L
    }

    pub fn read_ram_bank_l(&self) -> u8 {
        match self.mapper {
            Mapper::B_2001 => self.io_bus.borrow().open_bus(),
            Mapper::B_2003 => self.RAM_BANK_L,
        }
    }

    pub fn read_ram_bank_h(&self) -> u8 {
        match self.mapper {
            Mapper::B_2001 => self.io_bus.borrow().open_bus(),
            Mapper::B_2003 => self.RAM_BANK_H,
        }
    }

    pub fn read_rom_bank_0(&self) -> u8 {
        self.ROM_BANK_0_L
    }

    pub fn read_rom_bank_0_l(&self) -> u8 {
        match self.mapper {
            Mapper::B_2001 => self.io_bus.borrow().open_bus(),
            Mapper::B_2003 => self.ROM_BANK_0_L,
        }
    }

    pub fn read_rom_bank_0_h(&self) -> u8 {
        match self.mapper {
            Mapper::B_2001 => self.io_bus.borrow().open_bus(),
            Mapper::B_2003 => self.ROM_BANK_0_H,
        }
    }

    pub fn read_rom_bank_1(&self) -> u8 {
        self.ROM_BANK_1_L
    }

    pub fn read_rom_bank_1_l(&self) -> u8 {
        match self.mapper {
            Mapper::B_2001 => self.io_bus.borrow().open_bus(),
            Mapper::B_2003 => self.ROM_BANK_1_L,
        }
    }

    pub fn read_rom_bank_1_h(&self) -> u8 {
        match self.mapper {
            Mapper::B_2001 => self.io_bus.borrow().open_bus(),
            Mapper::B_2003 => self.ROM_BANK_1_H,
        }
    }

    pub fn write_ram_bank_l(&mut self, byte: u8) {
        self.RAM_BANK_L = byte;
    }

    pub fn write_ram_bank_h(&mut self, byte: u8) {
        self.RAM_BANK_H = byte;
    }

    pub fn write_rom_bank_0_l(&mut self, byte: u8) {
        self.ROM_BANK_0_L = byte;
    }

    pub fn write_rom_bank_0_h(&mut self, byte: u8) {
        self.ROM_BANK_0_H = byte;
    }

    pub fn write_rom_bank_1_l(&mut self, byte: u8) {
        self.ROM_BANK_1_L = byte;
    }

    pub fn write_rom_bank_1_h(&mut self, byte: u8) {
        self.ROM_BANK_1_H = byte;
    }
}