use cartridge::Mapper;
use soc::SoC;

pub mod soc;
pub mod bus;
pub mod dma;

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod cartridge;

#[allow(non_snake_case)]
pub mod cpu;

fn main() {
    let (color, sram, rom, mapper, rewrittable) = parse_rom("klonoa");
    let mut soc = SoC::new(color, sram, rom, mapper, rewrittable);
    soc.run(1000);
}

fn parse_rom(game: &str) -> (bool, Vec<u8>, Vec<u8>, Mapper, bool) {
    let rom = std::fs::read(format!("{}.ws", game)).unwrap();
    let footer = rom.last_chunk::<16>().unwrap();
    let color = footer[0x7] & 1 != 0;
    let (ram_size, rewrittable) = match footer[0xB] {
        0x00 => (0x0u32, true),
        0x01 | 0x02 => (0x08000, true),
        0x03 => (0x20000, true),
        0x04 => (0x40000, true),
        0x05 => (0x80000, true),
        0x10 => (0x080, false),
        0x20 => (0x800, false),
        0x50 => (0x400, false),
        _ => panic!("Unknown save type!")
    };
    let save = std::fs::read(format!("{}.sav", game)).or_else(|_| {Ok::<_, ()>(vec![0; ram_size as usize])}).unwrap();

    let mapper = match footer[0xD] {
        0 => Mapper::B_2001,
        1 => Mapper::B_2003,
        _ => panic!("Unknown mapper!"),
    };

    (color, save, rom, mapper, rewrittable)
}

#[macro_export]
macro_rules! assert_eq_hex {
    ($left:expr, $right:expr) => {
        let left_val = $left;
        let right_val = $right;
        assert!(
            left_val == right_val,
            "assertion `left == right` failed\n  left: 0x{:X}\n right: 0x{:X}",
            left_val, right_val,
        )
    };
}
