use std::{cell::RefCell, rc::Rc};

use crate::bus::{io_bus::{IOBus, IOBusConnection}, mem_bus::{MemBus, MemBusConnection}};

use super::{screen::ScreenElement, sprite::SpriteElement, PaletteFormat};

pub struct Display {
    mem_bus: Rc<RefCell<MemBus>>,
    io_bus: Rc<RefCell<IOBus>>,
}

impl MemBusConnection for Display {
    fn read_mem(&mut self, addr: u32) -> u8 {
        self.mem_bus.borrow_mut().read_mem(addr)
    }

    fn write_mem(&mut self, addr: u32, byte: u8) {
        self.mem_bus.borrow_mut().write_mem(addr, byte);
    }
}

impl IOBusConnection for Display {
    fn read_io(&mut self, addr: u16) -> u8 {
        self.io_bus.borrow_mut().read_io(addr)
    }

    fn write_io(&mut self, addr: u16, byte: u8) {
        self.io_bus.borrow_mut().write_io(addr, byte);
    }
}

impl Display {
    pub fn new(mem_bus: Rc<RefCell<MemBus>>, io_bus: Rc<RefCell<IOBus>>) -> Self {
        Self {
            mem_bus, io_bus,
        }
    }

    pub fn tick(&mut self) {
        todo!()
    }

    fn read_tile(&mut self, index: u16, format: PaletteFormat) -> [[u8; 8]; 8] {
        std::array::from_fn(|i| {
            match format {
                PaletteFormat::PLANAR_2BPP => {
                    let base = index as u32 + 0x2000 + i as u32 * 16;
                    let [plane0, plane1] = self.read_mem_16(base + i as u32 * 2).to_le_bytes();
                    std::array::from_fn(|j| {
                        let b = 7 - j as u8;
                        let b0 = (plane0 >> b) & 1;
                        let b1 = (plane1 >> b) & 1;
                        (b1 << 1) | b0
                    })
                }
                PaletteFormat::PLANAR_4BPP => {
                    let base = index as u32 + 0x4000 + i as u32 * 32;
                    let data = self.read_mem_32(base + i as u32 * 2);
                    let [plane0, plane1] = data.0.to_le_bytes();
                    let [plane2, plane3] = data.1.to_le_bytes();
                    std::array::from_fn(|j| {
                        let b = 7 - j as u8;
                            let b0 = (plane0 >> b) & 1;
                            let b1 = (plane1 >> b) & 1;
                            let b2 = (plane2 >> b) & 1;
                            let b3 = (plane3 >> b) & 1;
                            (b3 << 3) | (b2 << 2) | (b1 << 1) | b0
                    })
                }
                PaletteFormat::PACKED_4BPP => {
                    let base = index as u32 + 0x4000 + i as u32 * 32;
                    std::array::from_fn(|j| {
                        let bk_idx = i * 4 + j / 2;
                        let byte = self.read_mem(base + bk_idx as u32);
                        match j % 2 {
                            0 => byte >> 4,
                            1 => byte & 0x0F,
                            _ => unreachable!(),
                        }
                    })
                }
            }
        })
    }

    fn read_screen(&mut self, addr: u16, format: PaletteFormat) -> [ScreenElement; 1024] {
        let base = addr as u32;
        let color = self.io_bus.borrow_mut().color_mode();
        std::array::from_fn(|i| {
            let word = self.read_mem_16(base + i as u32 * 2);

            let vm = word & (1 << 15) != 0;
            let hm = word & (1 << 14) != 0;
            let palette = ((word >> 9) & 0x0F) as u8;
            let mut tile_idx = word & 0x001F;
            if color {
                tile_idx |= (word & 0x2000) >> 4;
            }

            ScreenElement::new(vm, hm, palette, self.read_tile(tile_idx, format))
        })
    }

    fn read_screen_element(&mut self, addr: u16, format: PaletteFormat) -> ScreenElement {
        let addr = addr as u32;
        let color = self.io_bus.borrow_mut().color_mode();

        let word = self.read_mem_16(addr);

        let vm = word & (1 << 15) != 0;
        let hm = word & (1 << 14) != 0;
        let palette = ((word >> 9) & 0x0F) as u8;
        let mut tile_idx = word & 0x001F;
        if color {
            tile_idx |= (word & 0x2000) >> 4;
        }

        ScreenElement::new(vm, hm, palette, self.read_tile(tile_idx, format))
    }

    fn read_sprite(&mut self, addr: u16, count: u8, format: PaletteFormat) -> Vec<SpriteElement> {
        let mut sprite = Vec::new();
        let base = addr as u32;

        for i in base..(base + (count & 0x7F) as u32) {
            let sprite_idx = i % 128;
            let element_addr = base + sprite_idx * 4;

            let (word, coords) = self.read_mem_32(element_addr);
            let vm = word & (1 << 15) != 0;
            let hm = word & (1 << 14) != 0;
            let pr = word & (1 << 13) != 0;
            let ct = word & (1 << 12) != 0;
            let palette = ((word >> 9) & 0x07) as u8;
            let tile = self.read_tile(word & 0x1F, format);
            let [x, y] = coords.to_le_bytes();

            sprite.push(SpriteElement::new(vm, hm, pr, ct, palette, tile, x, y));
        }

        sprite
    }

    fn get_monochrome_palette(&mut self, index: u8) -> [(u8, u8, u8); 4] {
        let (lo, hi) = self.read_io_16(0x20 + (index as u16) * 2);
        let (c0, c1) = (lo & 0x07, (lo >> 4) & 0x07);
        let (c2, c3) = (hi & 0x07, (hi >> 4) & 0x07);

        std::array::from_fn(|i| {
            let index = [c0, c1, c2, c3][i];
            let (port, shift) = (index / 2, index % 2);
            let color_raw = (self.read_io(0x1C + port as u16) >> shift * 4) & 0x0F;
            let color = 0xFF - 0x11 * color_raw;

            (color, color, color)
        })
    }

    fn get_color_palette(&mut self, index: u8) -> [(u8, u8, u8); 16] {
        let base = 0x0FE00 + (index as u32) * 0x20;

        std::array::from_fn(|i| {
            let word = self.read_mem_16(base + i as u32 * 2);
            ((word & 0x0F) as u8, ((word >> 4) & 0x0F) as u8, ((word >> 8) & 0x0F) as u8)
        })
    }
}