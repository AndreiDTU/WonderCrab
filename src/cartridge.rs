use crate::bus::io_bus::IOBus;

pub mod cart_ports;

pub enum Mapper {
    B_2001,
    B_2003,
}

pub struct Cartridge {
    sram: Vec<u8>,
    rom: Vec<u8>,

    mapper: Mapper,

    RAM_BANK_L: u8,
    RAM_BANK_H: u8,

    ROM_BANK_0_L: u8,
    ROM_BANK_0_H: u8,
    ROM_BANK_1_L: u8,
    ROM_BANK_1_H: u8,
    LINEAR_ADDR_OFF: u8,

    rewrittable: bool,
}

impl Cartridge {
    pub fn new(mapper: Mapper, sram: Vec<u8>, rom: Vec<u8>, rewrittable: bool) -> Self {
        Self {
            sram, rom, mapper,
            RAM_BANK_L: 0xFF, RAM_BANK_H: 0xFF,
            ROM_BANK_0_L: 0xFF, ROM_BANK_0_H: 0xFF,
            ROM_BANK_1_L: 0xFF, ROM_BANK_1_H: 0xFF,
            LINEAR_ADDR_OFF: 0xFF,
            rewrittable,
        }
    }

    pub fn read_sram(&self, addr: u32) -> u8 {
        let hi = match self.mapper {
            Mapper::B_2001 => self.RAM_BANK_L as u32,
            Mapper::B_2003 => u16::from_le_bytes([self.RAM_BANK_L, self.RAM_BANK_H]) as u32,
        };
        let lo = addr & 0xFFFF;

        let offset = ((hi << 16) | lo) % self.sram.len() as u32;

        if offset as usize > self.sram.len() {
            IOBus::open_bus()
        } else {
            self.sram[offset as usize]
        }
    }

    pub fn write_sram(&mut self, addr: u32, byte: u8) {
        if self.rewrittable {
            let hi = match self.mapper {
                Mapper::B_2001 => self.RAM_BANK_L as u32,
                Mapper::B_2003 => u16::from_le_bytes([self.RAM_BANK_L, self.RAM_BANK_H]) as u32,
            };
            let lo = addr & 0xFFFF;

            let offset = ((hi << 16) | lo) % self.sram.len() as u32;
            
            if !(offset as usize > self.sram.len()) {
                self.sram[offset as usize] = byte;
            }
        }
    }

    pub fn read_rom_0(&self, addr: u32) -> u8 {
        let hi = match self.mapper {
            Mapper::B_2001 => self.ROM_BANK_0_L as u32,
            Mapper::B_2003 => u16::from_le_bytes([self.ROM_BANK_0_L, self.ROM_BANK_0_H]) as u32,
        };
        let lo = addr & 0xFFFF;

        let offset = ((hi << 16) | lo) % self.rom.len() as u32;

        self.rom[offset as usize]
    }

    pub fn read_rom_1(&self, addr: u32) -> u8 {
        let hi = match self.mapper {
            Mapper::B_2001 => self.ROM_BANK_1_L as u32,
            Mapper::B_2003 => u16::from_le_bytes([self.ROM_BANK_1_L, self.ROM_BANK_1_H]) as u32,
        };
        let lo = addr & 0xFFFF;

        let offset = ((hi << 16) | lo) % self.rom.len() as u32;

        self.rom[offset as usize]
    }

    pub fn read_rom_ex(&self, addr: u32) -> u8 {
        let hi = (self.LINEAR_ADDR_OFF as u32) << 20;
        let offset = (hi | addr) % self.rom.len() as u32;

        self.rom[offset as usize]
    }

    pub fn test_build() -> Self {
        Self::new(Mapper::B_2001, vec![0; 0x100000], vec![0; 0x100000], true)
    }
}