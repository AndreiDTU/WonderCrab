use crate::assert_eq_hex;

use super::*;

impl SoC {
    pub fn set_wram(&mut self, wram: Vec<u8>) {
        for i in 0..wram.len() {
            self.mem_bus.borrow_mut()[i] = wram[i];
        }
    }

    pub fn set_io(&mut self, io: Vec<u8>) {
        for i in 0..io.len() {
            self.io_bus.borrow_mut().write_io(i as u16, io[i]);
        }
    }

    pub fn get_cpu(&mut self) -> &mut V30MZ {
        &mut self.cpu
    }

    pub fn get_wram(&mut self) -> Rc<RefCell<MemBus>> {
        Rc::clone(&self.mem_bus)
    }

    pub fn tick_cpu_no_cycles(&mut self) {
        self.cpu.tick_ignore_cycles();
    }
    
}

#[test]
fn test_io_open_bus() {
    let mut soc = SoC::test_build();
    assert_eq_hex!(soc.read_io(0x100), 0x90);
    assert_eq_hex!(soc.read_io(0x1B9), 0x90);
}

#[test]
fn test_map_word() {
    let soc = SoC::test_build();

    for i in 0..1024 {
        soc.mem_bus.borrow_mut().write_mem_16(i * 2, (i as u16) ^ 0x1234);
    }

    assert_eq_hex!(soc.display.fetch_map_word(0, 0, 0), 0x1234);
    assert_eq_hex!(soc.display.fetch_map_word(0, 0, 1), (1 as u16)^0x1234);
    assert_eq_hex!(soc.display.fetch_map_word(0, 1, 0), (32 as u16)^0x1234);
}

#[test]
fn test_decode_2bpp_tile() {
    let mut soc = SoC::test_build();

    let ti = 3;
    let tile_base = 0x2000 + (ti as u32) * 16;
    for row in 0..8 {
        let addr = tile_base + (row as u32) * 2;
        soc.write_mem_16(addr, 0x55AA);
    }

    let bytes = soc.mem_bus.borrow_mut().read_mem_16(tile_base);
    assert_eq_hex!(bytes, 0b0101_0101_1010_1010);

    let tile = soc.display.decode_2bpp_tile(ti);
    assert_eq!(tile[0][0], 1);
    assert_eq!(tile[0][1], 2);
}

#[test]
fn test_monochrome_palette_entries() {
    let mut soc = SoC::test_build();

    // Pick palette #1 (ports 0x22/0x23)
    // We'll make its entries [c0,c1,c2,c3] = [0,1,2,3]
    // lo = c1<<4 | c0 = (1<<4)|0 = 0x10
    // hi = c3<<4 | c2 = (3<<4)|2 = 0x32
    soc.write_io(0x22, 0x10);
    soc.write_io(0x23, 0x32);

    soc.write_io(0x1C, 0x10);
    soc.write_io(0x1D, 0x32);

    let pal = soc.display.force_get_monochrome_palette(1);
    // entry 0: raw=0 → color=0xFF-0x11*0 = 0xFF
    assert_eq!(pal[0], (0xFF,0xFF,0xFF));
    // entry 1: raw=1 → 0xFF-0x11=0xEE
    assert_eq!(pal[1], (0xEE,0xEE,0xEE));
    // entry 2: raw=2 → 0xFF-0x22=0xDD
    assert_eq!(pal[2], (0xDD,0xDD,0xDD));
    // entry 3: raw=3 → 0xFF-0x33=0xCC
    assert_eq!(pal[3], (0xCC,0xCC,0xCC));
}