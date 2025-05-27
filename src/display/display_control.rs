use std::{cell::RefCell, rc::Rc};

use crate::bus::{io_bus::IOBus, mem_bus::{MemBus, MemBusConnection}};

use super::{screen::ScreenElement, PaletteFormat};

pub struct Display {
    mem_bus: Rc<RefCell<MemBus>>,
    io_bus: Rc<RefCell<IOBus>>,
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
                    let [plane0, plane1] = self.mem_bus.borrow_mut().read_mem_16(base + i as u32 * 2).to_le_bytes();
                    std::array::from_fn(|j| {
                        let b = 7 - j as u8;
                        let b0 = (plane0 >> b) & 1;
                        let b1 = (plane1 >> b) & 1;
                        (b1 << 1) | b0
                    })
                }
                PaletteFormat::PLANAR_4BPP => {
                    let base = index as u32 + 0x4000 + i as u32 * 32;
                    let data = self.mem_bus.borrow_mut().read_mem_32(base + i as u32 * 2);
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
                        let byte = self.mem_bus.borrow_mut().read_mem(base + bk_idx as u32);
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
            let word = self.mem_bus.borrow_mut().read_mem_16(base + i as u32 * 2);

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
}