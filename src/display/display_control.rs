use std::{cell::RefCell, rc::Rc};

use crate::bus::{io_bus::{IOBus, IOBusConnection}, mem_bus::{MemBus, MemBusConnection}};

use super::{screen::ScreenElement, sprite::SpriteElement, PaletteFormat};

pub struct Display {
    mem_bus: Rc<RefCell<MemBus>>,
    io_bus: Rc<RefCell<IOBus>>,

    format: PaletteFormat,

    screen_1_base: u16,
    screen_2_base: u16,
    sprite_base: u16,

    screen_1_elements: [[ScreenElement; 32]; 32],
    screen_2_elements: [[ScreenElement; 32]; 32],

    screen_1_tiles: [[[[u8; 8]; 8]; 32]; 32],
    screen_2_tiles: [[[[u8; 8]; 8]; 32]; 32],

    screen_1_pixels: [[(u8, u8, u8); 256]; 256],
    screen_2_pixels: [[(u8, u8, u8); 256]; 256],

    sprites: [SpriteElement; 128],
    sprite_tiles: [[[u8; 8]; 8]; 128],
    sprite_counter: u8, finished_sprites: bool,

    pub lcd: [(u8, u8, u8); 224 * 144],

    scanline: u8,
    cycle: u8,
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
        let format = io_bus.borrow_mut().pallete_format();
        Self {
            mem_bus, io_bus,
            scanline: 0, cycle: 0,

            format,
            screen_1_base: 0, screen_2_base: 0, sprite_base: 0,
            screen_1_elements: [[ScreenElement::dummy(); 32]; 32], screen_2_elements: [[ScreenElement::dummy(); 32]; 32],
            screen_1_tiles: [[[[0; 8]; 8]; 32]; 32], screen_2_tiles: [[[[0; 8]; 8]; 32]; 32],
            screen_1_pixels: [[(0, 0, 0); 256]; 256], screen_2_pixels: [[(0, 0, 0); 256]; 256],
            sprites: [SpriteElement::dummy(); 128], sprite_tiles: [[[0; 8]; 8]; 128], sprite_counter: 0, finished_sprites: false,
            lcd: [(0, 0, 0); 224 * 144],
        }
    }

    pub fn tick(&mut self) {
        self.format = self.io_bus.borrow_mut().pallete_format();

        if self.cycle % 2 == 1 && self.sprite_counter > 0 {
            self.sprites[self.cycle as usize / 2] = self.read_sprite(self.sprite_base + self.cycle as u16 * 4, self.format);
            self.sprite_counter -= 1;
        }

        match self.cycle {
            // Find screen 1's tile and element data
            0 => {
                self.get_screen_1_base();
                self.get_sprite_base();
                self.get_sprite_counter();
                self.finished_sprites = false;
                self.screen_1_elements[self.scanline as usize][0] = self.read_screen_element(self.screen_1_base);
            }
            1..=63 => {
                if self.cycle % 2 == 1 {
                    self.screen_1_tiles[self.scanline as usize][self.cycle as usize / 2] = self.read_tile(self.screen_1_elements[self.scanline as usize][self.cycle as usize / 2].tile_idx, self.format);
                } else {
                    self.screen_1_elements[self.scanline as usize][self.cycle as usize / 2] = self.read_screen_element(self.screen_1_base.wrapping_add(self.cycle as u16 * 2))
                }
            }

            // Find screen 2's tile and element data
            65 => {self.get_screen_2_base()}
            66..=129 => {
                if self.cycle % 2 == 1 {
                    self.screen_2_tiles[self.scanline as usize][(self.cycle - 66) as usize / 2] = self.read_tile(self.screen_2_elements[self.scanline as usize][(self.cycle - 66) as usize / 2].tile_idx, self.format);
                } else {
                    self.screen_2_elements[self.scanline as usize][(self.cycle - 66) as usize / 2] = self.read_screen_element(self.screen_2_base.wrapping_add((self.cycle - 66) as u16 * 2))
                }
            }

            // Find sprite data
            158 => self.fetch_sprite_tile(1),
            160 => self.fetch_sprite_tile(2),
            162 => self.fetch_sprite_tile(3),
            166 => self.fetch_sprite_tile(4),
            168 => self.fetch_sprite_tile(5),
            170 => self.fetch_sprite_tile(6),
            174 => self.fetch_sprite_tile(7),
            176 => self.fetch_sprite_tile(8),
            178 => self.fetch_sprite_tile(9),
            182 => self.fetch_sprite_tile(10),
            184 => self.fetch_sprite_tile(11),
            186 => self.fetch_sprite_tile(12),
            190 => self.fetch_sprite_tile(13),
            192 => self.fetch_sprite_tile(14),
            194 => self.fetch_sprite_tile(15),
            198 => self.fetch_sprite_tile(16),
            200 => self.fetch_sprite_tile(17),
            202 => self.fetch_sprite_tile(18),
            206 => self.fetch_sprite_tile(19),
            208 => self.fetch_sprite_tile(20),
            210 => self.fetch_sprite_tile(21),
            214 => self.fetch_sprite_tile(22),
            216 => self.fetch_sprite_tile(23),
            218 => self.fetch_sprite_tile(24),
            222 => self.fetch_sprite_tile(25),
            224 => self.fetch_sprite_tile(26),
            226 => self.fetch_sprite_tile(27),
            230 => self.fetch_sprite_tile(28),
            232 => self.fetch_sprite_tile(29),
            234 => self.fetch_sprite_tile(30),
            238 => self.fetch_sprite_tile(31),
            240 => self.fetch_sprite_tile(32),

            255 => {
                self.cycle = 0;
                self.scanline += 1;
            }
            _ => {}
        }

        // Display pixels of previous scanline
        if self.scanline != 0 && self.scanline < 144 && self.cycle < 224 {
            self.lcd[(self.cycle as usize) + (self.scanline as usize) * 144] = (0, 0, 0);
        }
    }

    fn get_screen_1_base(&mut self) {
        self.screen_1_base = ((self.io_bus.borrow_mut().read_io(0x07) & 0x0F) as u16) << 10;
        if !self.io_bus.borrow_mut().color_mode() {self.screen_1_base &= 0x3800}
    }

    fn get_screen_2_base(&mut self) {
        self.screen_2_base = (((self.io_bus.borrow_mut().read_io(0x07) >> 4) & 0x0F) as u16) << 10;
        if !self.io_bus.borrow_mut().color_mode() {self.screen_2_base &= 0x3800}
    }

    fn get_sprite_base(&mut self) {
        self.sprite_base = (self.read_io(0x04) as u16) << 9;
        if !self.io_bus.borrow_mut().color_mode() {self.sprite_base &= 0x3E00}
    }

    fn get_sprite_counter(&mut self) {
        self.sprite_counter = self.read_io(0x06) & 0x7F;
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

    fn read_screen_element(&mut self, addr: u16) -> ScreenElement {
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

        ScreenElement::new(vm, hm, palette, tile_idx)
    }

    fn read_sprite(&mut self, addr: u16, format: PaletteFormat) -> SpriteElement {
        let base = addr as u32;

        let (word, coords) = self.read_mem_32(base);
        let vm = word & (1 << 15) != 0;
        let hm = word & (1 << 14) != 0;
        let pr = word & (1 << 13) != 0;
        let ct = word & (1 << 12) != 0;
        let palette = ((word >> 9) & 0x07) as u8;
        let [x, y] = coords.to_le_bytes();

        SpriteElement::new(vm, hm, pr, ct, palette, word & 0x1F, x, y)
    }

    fn fetch_sprite_tile(&mut self, index: u8) {
        if self.finished_sprites {return}

        if let Some(sprite) = self.sprites
            .iter()
            .filter(|s| {
                (s.y..s.y+8).contains(&self.scanline)
            }).collect::<Vec<&SpriteElement>>()
            .get(128 - (index as usize)) {
            self.sprite_tiles[128 - index as usize] = self.read_tile(sprite.tile_idx, self.format);
        } else {
            self.finished_sprites = true;
        }
    }

    fn fetch_pixel_color(&mut self, palette: u8, pixel: u8) -> Option<(u8, u8, u8)> {
        match self.format {
            PaletteFormat::PLANAR_2BPP => {
                if palette < 4 && pixel == 0 {
                    return None;
                }

                Some(if self.io_bus.borrow_mut().color_mode() {
                    self.get_color_palette(palette)[pixel as usize]
                } else {
                    self.get_monochrome_palette(palette)[pixel as usize]
                })
            },
            PaletteFormat::PLANAR_4BPP | PaletteFormat::PACKED_4BPP => {
                if pixel == 0 {
                    return None;
                }

                Some(self.get_color_palette(palette)[pixel as usize])
            },
        }
    }

    fn resolve_pixel_color(&mut self) -> (u8, u8, u8) {
        let (lo, hi) = self.io_bus.borrow_mut().read_io_16(0x00);
        let word = u16::from_le_bytes([lo, hi]);

        let scr1 = word & 1 != 0;
        let scr2 = (word >> 1) != 0;
        let spr = (word >> 2) != 0;
        let sprwe = (word >> 3) != 0;
        let s2we = (word >> 4) != 0;
        let s2wc = (word >> 5) != 0;

        let bg_color = if self.io_bus.borrow_mut().color_mode() {
            self.get_color_palette((word >> 12) as u8)[((word >> 8) & 0x0F) as usize]
        } else {
            let index = ((word >> 8) & 0x7) as u8;
            let (port, shift) = (index / 2, index % 2);
            let color_raw = (self.read_io(0x1C + port as u16) >> shift * 4) & 0x0F;
            let color = 0xFF - 0x11 * color_raw;

            (color, color, color)   
        };

        todo!()
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