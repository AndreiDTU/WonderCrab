use crate::bus::io_bus::IOBus;

/// Various getter and setter functions meant to be used by the I/O bus
pub mod cart_ports;

/// The mapper chips contained within WonderSwan cartridges
#[derive(PartialEq)]
pub enum Mapper {
    /// Bandai 2001 mapper
    B_2001,
    /// Bandai 2003 mapper (not properly supported)
    B_2003,
}

/// The cartridge struct
pub struct Cartridge {
    /// The cartridge SRAM, may be empty
    pub(crate) sram: Vec<u8>,
    /// The contents of the ROM
    rom: Vec<u8>,

    /// The mapper chip
    mapper: Mapper,

    /// Low byte of the RAM bank, or the entire bank if the mapper is 2001
    RAM_BANK_L: u8,
    /// High byte of the RAM bank, ignored on mapper 2001
    RAM_BANK_H: u8,

    /// Low byte of ROM bank 0, or the entire bank if the mapper is 2001
    ROM_BANK_0_L: u8,
    /// High byte of ROM bank 0, ignored mapper on 2001
    ROM_BANK_0_H: u8,
    /// Low byte of ROM bank 1, or the entire bank if the mapper is 2001
    ROM_BANK_1_L: u8,
    /// High byte of ROM bank 1, ignored mapper on 2001
    ROM_BANK_1_H: u8,
    /// Offset into the extended addressing space
    LINEAR_ADDR_OFF: u8,

    /// Whether or not the cartridge contains SRAM
    rewrittable: bool,
}

impl Cartridge {
    /// Returns a new cartridge, requires a mapper, SRAM, ROM and the `rewrittable` boolean, all other fields initialized to 0xFF
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

    /// Reads the SRAM at the index formed by combining the provided address with the RAM bank
    pub fn read_sram(&self, addr: u32) -> u8 {
        if self.sram.len() == 0 {
            return IOBus::open_bus();
        }
        
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

    /// Writes a byte to the SRAM at the index formed by combining the provided address with the RAM bank
    pub fn write_sram(&mut self, addr: u32, byte: u8) {
        if self.sram.len() == 0 {return}
        if self.rewrittable {
            let hi = match self.mapper {
                Mapper::B_2001 => self.RAM_BANK_L as u32,
                Mapper::B_2003 => u16::from_le_bytes([self.RAM_BANK_L, self.RAM_BANK_H]) as u32,
            };
            let lo = addr & 0xFFFF;

            let offset = ((hi << 16) | lo) % self.sram.len() as u32;

        // print!("CART SRAM_OFFSET: {:07X}", offset);
            
            if !(offset as usize > self.sram.len()) {
                self.sram[offset as usize] = byte;
            }
        }
    }

    /// Reads the ROM at the index formed by combining the provided address with the ROM bank 0
    pub fn read_rom_0(&self, addr: u32) -> u8 {
        let hi = match self.mapper {
            Mapper::B_2001 => self.ROM_BANK_0_L as u32,
            Mapper::B_2003 => u16::from_le_bytes([self.ROM_BANK_0_L, self.ROM_BANK_0_H]) as u32,
        };
        let lo = addr & 0xFFFF;

        let offset = ((hi << 16) | lo) % self.rom.len() as u32;

        // print!("CART ROM0_OFFSET: {:07X}", offset);

        self.rom[offset as usize]
    }

    /// Reads the ROM at the index formed by combining the provided address with the ROM bank 1
    pub fn read_rom_1(&self, addr: u32) -> u8 {
        let hi = match self.mapper {
            Mapper::B_2001 => self.ROM_BANK_1_L as u32,
            Mapper::B_2003 => u16::from_le_bytes([self.ROM_BANK_1_L, self.ROM_BANK_1_H]) as u32,
        };
        let lo = addr & 0xFFFF;

        let offset = ((hi << 16) | lo) % self.rom.len() as u32;

        // print!("CART ROM1_OFFSET: {:07X}", offset);

        self.rom[offset as usize]
    }

    /// Reads the ROM at the index formed by combining the provided address with the extended range offset
    pub fn read_rom_ex(&self, addr: u32) -> u8 {
        let addr = addr & 0xFFFFF;
        let hi = (self.LINEAR_ADDR_OFF as u32) << 20;
        let offset = (hi | addr) % self.rom.len() as u32;

        // print!("CART EX_OFFSET: {:07X}", offset);

        self.rom[offset as usize]
    }

    /// A test build used during tests or if the user does not provide a ROM
    pub fn test_build() -> Self {
        Self::new(Mapper::B_2001, vec![0; 0x100000], vec![0; 0x100000], true)
    }
}