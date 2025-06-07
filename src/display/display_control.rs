use std::{cell::RefCell, rc::Rc};

use crate::bus::{io_bus::{IOBus, IOBusConnection}, mem_bus::{MemBus, MemBusConnection}};

use super::{screen::ScreenElement, sprite::SpriteElement, PaletteFormat};

pub struct Display {
    mem_bus: Rc<RefCell<MemBus>>,
    io_bus: Rc<RefCell<IOBus>>,

    format: PaletteFormat,
    color: bool,

    screen_1_base: u16,
    screen_2_base: u16,
    sprite_base: u16,

    screen_1_elements: [[ScreenElement; 32]; 32],
    screen_2_elements: [[ScreenElement; 32]; 32],

    screen_1_tiles: [[[[u8; 8]; 8]; 32]; 32],
    screen_2_tiles: [[[[u8; 8]; 8]; 32]; 32],

    screen_2_pixels: Box<[[Option<(u8, u8, u8)>; 256]; 256]>,

    sprite_table: [SpriteElement; 128],
    sprite_tiles: [[[u8; 8]; 8]; 128],
    sprite_pixels: Box<[[Option<(u8, u8, u8)>; 256]; 256]>,
    sprite_counter: u8, finished_sprites: bool,

    shared_lcd: Rc<RefCell<[u8; 3 * 224 * 144]>>,
    lcd: Box<[u8; 3 * 224 * 144]>,

    scanline: u8,
    cycle: u8,

    color_map: [[Option<(u8, u8, u8)>; 16]; 16],
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
    pub fn new(mem_bus: Rc<RefCell<MemBus>>, io_bus: Rc<RefCell<IOBus>>, shared_lcd: Rc<RefCell<[u8; 3 * 224 * 144]>>) -> Self {
        let format = io_bus.borrow_mut().palette_format();
        let color = io_bus.borrow_mut().color_mode();
        Self {
            mem_bus, io_bus,
            scanline: 0, cycle: 0,

            format,
            color,
            screen_1_base: 0, screen_2_base: 0, sprite_base: 0,
            
            screen_1_elements: [[ScreenElement::dummy(); 32]; 32], screen_2_elements: [[ScreenElement::dummy(); 32]; 32],
            screen_1_tiles: [[[[0; 8]; 8]; 32]; 32],  screen_2_tiles: [[[[0; 8]; 8]; 32]; 32],
            screen_2_pixels: Box::new([[None; 256]; 256]),

            sprite_table: [SpriteElement::dummy(); 128], sprite_tiles: [[[0; 8]; 8]; 128], sprite_pixels: Box::new([[None; 256]; 256]),
            sprite_counter: 0, finished_sprites: false,
            
            shared_lcd, lcd: Box::new([0; 3 * 224 * 144]),
            color_map: [[None; 16]; 16]
        }
    }

    pub fn tick(&mut self) {
        self.color = self.io_bus.borrow_mut().color_mode();
        self.format = self.io_bus.borrow_mut().palette_format();

        let (x, y) = (self.cycle as usize, self.scanline as usize);

        match self.cycle {
            // Find screen 1's tile and element data
            0 => {
                if self.scanline == 0 {
                    self.get_screen_1_base();
                    self.get_sprite_base();
                    self.get_sprite_counter();
                }
                self.generate_color_map();
                self.finished_sprites = false;

                let row = y >> 3;
                let address = self.screen_1_base | ((row as u16) << 6);
                self.screen_1_elements[row][0] = self.read_screen_element(address);
            }
            1..=63 => {
                let (row, col) = (y >> 3, x / 2);
                if self.cycle % 2 == 1 {
                    self.screen_1_tiles[row][col] = self.read_tile(self.screen_1_elements[row][col].tile_idx, self.format);
                } else {
                    let address = self.screen_1_base | ((row as u16) << 6) | (col as u16 * 2);
                    self.screen_1_elements[row][col] = self.read_screen_element(address);
                }
            }

            // Find screen 2's tile and element data
            65 => {
                if self.scanline == 0 {self.get_screen_2_base()};
                self.screen_2_elements[y >> 3][0] = self.read_screen_element(self.screen_2_base);
            }
            66..=129 => {
                let (row, col) = (y >> 3, (x - 66) / 2);
                if self.cycle % 2 == 1 {
                    self.screen_2_tiles[row][col] = self.read_tile(self.screen_2_elements[row][col].tile_idx, self.format);
                } else {
                    let address = self.screen_2_base | ((row as u16) << 6) | (col as u16 * 2);
                    self.screen_2_elements[row][col] = self.read_screen_element(address);
                }
            }

            /*
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
            */

            225 => {
                self.io_bus.borrow_mut().hblank();
            }

            255 => {
                self.scanline += 1;
                self.io_bus.borrow_mut().set_lcd_line(self.scanline);
            }
            _ => {}
        }

        if self.scanline == 0 && self.cycle < 224 {
            self.overlay_pixels(self.cycle, 144 - 1);
        }

        // Display pixels of previous scanline
        if (1..=143).contains(&self.scanline) && self.cycle < 224 {
            self.overlay_pixels(self.cycle, self.scanline - 1);
        }

        if self.scanline == 144 {
            if self.cycle == 0 {
                self.sprite_table = [SpriteElement::dummy(); 128];
                self.sprite_tiles = [[[0; 8]; 8]; 128];
            }
            if self.sprite_counter > 0 && self.cycle % 2 == 0 {
                let sprite_start = self.read_io(0x05) & 0x7F;
                let sprite_idx = (self.cycle / 2).wrapping_add(sprite_start) & 0x7F;
                let sprite_addr = self.sprite_base.wrapping_add(sprite_idx as u16 * 4);
                let sprite = self.read_sprite(sprite_addr);
                self.sprite_table[sprite_idx as usize] = sprite;
                self.sprite_tiles[sprite_idx as usize] = self.read_tile(sprite.tile_idx, self.format);
                self.sprite_counter -= 1;
            }
            if self.cycle == 255 {
                *self.shared_lcd.borrow_mut() = *self.lcd;
                self.io_bus.borrow_mut().vblank();
            }
        }

        if self.scanline == 255 {
            self.scanline = 0;
            self.io_bus.borrow_mut().set_lcd_line(self.scanline);
        }

        self.cycle = self.cycle.wrapping_add(1);
    }

    fn get_screen_1_base(&mut self) {
        self.screen_1_base = ((self.io_bus.borrow_mut().read_io(0x07) & 0x0F) as u16) << 11;
        if !self.color {self.screen_1_base &= 0x3800}
        // println!("Screen 1 base: {:014X}", self.screen_1_base);
    }

    fn get_screen_2_base(&mut self) {
        self.screen_2_base = (((self.io_bus.borrow_mut().read_io(0x07) >> 4) & 0x0F) as u16) << 11;
        if !self.color {self.screen_2_base &= 0x3800}
    }

    fn get_sprite_base(&mut self) {
        self.sprite_base = ((self.read_io(0x04) & 0x3F) as u16) << 9;
        if !self.color {self.sprite_base &= 0x3E00}
    }

    fn get_sprite_counter(&mut self) {
        self.sprite_counter = self.read_io(0x06) & 0x7F;
    }

    fn read_tile(&mut self, index: u16, format: PaletteFormat) -> [[u8; 8]; 8] {
        std::array::from_fn(|row| {
            match format {
                PaletteFormat::PLANAR_2BPP => {
                    let base = 0x2000 + (index as u32) * 16;
                    let [plane0, plane1] = self.read_mem_16(base + (row as u32 * 2)).to_le_bytes();
                    std::array::from_fn(|col| {
                        let b = 7 - col as u8;
                        let b0 = (plane0 >> b) & 1;
                        let b1 = (plane1 >> b) & 1;
                        (b1 << 1) | b0
                    })
                }
                PaletteFormat::PLANAR_4BPP => {
                    let base = 0x4000 + (index as u32) * 32;
                    let data = self.read_mem_32(base + (row as u32 * 4));
                    let [plane0, plane1] = data.0.to_le_bytes();
                    let [plane2, plane3] = data.1.to_le_bytes();
                    std::array::from_fn(|col| {
                        let b = 7 - col as u8;
                            let b0 = (plane0 >> b) & 1;
                            let b1 = (plane1 >> b) & 1;
                            let b2 = (plane2 >> b) & 1;
                            let b3 = (plane3 >> b) & 1;
                            (b3 << 3) | (b2 << 2) | (b1 << 1) | b0
                    })
                }
                PaletteFormat::PACKED_4BPP => {
                    let base = 0x4000 + (index as u32) * 32;
                    std::array::from_fn(|col| {
                        let bk_idx = row * 4 + col / 2;
                        let byte = self.read_mem(base + bk_idx as u32);
                        match col % 2 {
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
        let color = self.color;

        let word = self.read_mem_16(addr);

        let vm = word & (1 << 15) != 0;
        let hm = word & (1 << 14) != 0;
        let palette = ((word >> 9) & 0x0F) as u8;
        let mut tile_idx = word & 0x01FF;
        if color {
            tile_idx |= (word & 0x2000) >> 4;
        }

        ScreenElement::new(vm, hm, palette, tile_idx)
    }

    fn read_sprite(&mut self, addr: u16) -> SpriteElement {
        let base = addr as u32;

        let (word, coords) = self.read_mem_32(base);
        let vm = word & (1 << 15) != 0;
        let hm = word & (1 << 14) != 0;
        let pr = word & (1 << 13) != 0;
        let ct = word & (1 << 12) != 0;
        let palette = ((word >> 9) & 0x07) as u8;
        let tile_idx = word & 0x1FF;
        let [y, x] = coords.to_le_bytes();

        SpriteElement::new(vm, hm, pr, ct, palette, tile_idx, x, y)
    }

    /*
    fn fetch_sprite_tile(&mut self, index: u8) {
        if self.finished_sprites {return}

        let filtered_indices: Vec<usize> = self.sprite_table
            .iter().enumerate()
            .filter(|(_, s)| {
                (s.y..s.y.wrapping_add(8)).contains(&self.scanline)
            }).map(|(i, _)| {i}).rev().collect();

        if let Some(sprite_idx) = filtered_indices.get(index as usize) {
            let sprite = self.sprite_table[*sprite_idx];
            self.sprite_tiles[*sprite_idx] = self.read_tile(sprite.tile_idx, self.format)
        } else {
            self.finished_sprites = true;
        }
    }
    */

    fn overlay_pixels(&mut self, x: u8, y: u8) {
        let (lo, hi) = self.io_bus.borrow_mut().read_io_16(0x00);
        let lcd_ctrl = u16::from_le_bytes([lo, hi]);

        let scr1  = lcd_ctrl & 1 != 0;
        let scr2  = (lcd_ctrl >> 1) & 1 != 0;
        let spr   = (lcd_ctrl >> 2) & 1 != 0;
        let sprwe = (lcd_ctrl >> 3) & 1 != 0;
        let s2wc  = (lcd_ctrl >> 4) & 1 != 0;
        let s2we  = (lcd_ctrl >> 5) & 1 != 0;
        
        if scr2 {
            self.screen_2_pixels[y as usize][x as usize] = self.apply_scr2_window(s2we, s2wc, x as u8, y as u8);
        } else {
            self.screen_2_pixels[y as usize][x as usize] = None;
        }

        self.sprite_pixels[y as usize][x as usize] = None;
        if spr {
            let filtered_indices: Vec<usize> = match (scr2, sprwe) {
                (true, true) => {
                    let (x1, x2) = (self.read_io(0x0C), self.read_io(0x0E));
                    if x2 < x1 {Vec::new()} else {
                        let (y1, y2) = (self.read_io(0x0D), self.read_io(0x0F));
                        if y2 < y1 {Vec::new()} else {
                            self.sprite_table.iter().enumerate()
                                .filter(|(_, s)| {(s.x..s.x.wrapping_add(8)).contains(&x)})
                                .filter(|(_, s)| {(s.y..s.y.wrapping_add(8)).contains(&y)})
                                .filter(|(_, s)| {s.ct != (x1..=x2).contains(&x) && s.ct != (y1..=y2).contains(&y)})
                                .filter(|(_, s)| {s.pr || self.screen_2_pixels[y as usize][x as usize].is_none()})
                                .map(|(i, _)| {i}).collect()
                        }
                    }
                }
                (true, false) => self.sprite_table.iter().enumerate()
                        .filter(|(_, s)| {(s.x..s.x.wrapping_add(8)).contains(&x)})
                        .filter(|(_, s)| {(s.y..s.y.wrapping_add(8)).contains(&y)})
                        .filter(|(_, s)| {s.pr || self.screen_2_pixels[y as usize][x as usize] == None})
                        .map(|(i, _)| {i}).collect(),
                (false, true) => {
                    let (x1, x2) = (self.read_io(0x0C), self.read_io(0x0E));
                    if x2 < x1 {Vec::new()} else {
                        let (y1, y2) = (self.read_io(0x0D), self.read_io(0x0F));
                        if y2 < y1 {Vec::new()} else {
                            self.sprite_table.iter().enumerate()
                                .filter(|(_, s)| {(s.x..s.x.wrapping_add(8)).contains(&x)})
                                .filter(|(_, s)| {(s.y..s.y.wrapping_add(8)).contains(&y)})
                                .filter(|(_, s)| {s.ct != (x1..=x2).contains(&x) && s.ct != (y1..=y2).contains(&y)})
                                .map(|(i, _)| {i}).collect()
                        }
                    }
                }
                (false, false) => self.sprite_table.iter().enumerate()
                    .filter(|(_, s)| {(s.x..s.x.wrapping_add(8)).contains(&x)})
                    .filter(|(_, s)| {(s.y..s.y.wrapping_add(8)).contains(&y)})
                    .map(|(i, _)| {i}).collect(),
            };

            for idx in filtered_indices {
                let sprite = &self.sprite_table[idx];
                let (dx, dy) = (x - sprite.x, y - sprite.y);
                let (dx, dy) = (
                    if sprite.hm {7 - dx} else {dx},
                    if sprite.vm {7 - dy} else {dy},
                );
                let raw_px = self.sprite_tiles[idx][dy as usize][dx as usize];
                let palette = sprite.palette;
                if let Some(color) = self.color_map[palette as usize + 8][raw_px as usize] {
                    self.sprite_pixels[y as usize][x as usize] = Some(color);
                    break;
                }
            }
        }

        let pixel =
            if let Some(spr_px) = self.sprite_pixels[y as usize][x as usize] {spr_px} 
            else if let Some(scr2_px) = self.screen_2_pixels[y as usize][x as usize] {scr2_px}
            else {
                if let Some(scr1_px) = 
                    if scr1 {
                        let scroll_x = self.read_io(0x10);
                        let scroll_y = self.read_io(0x11);

                        let mut pixel = (x.wrapping_add(scroll_x), y.wrapping_add(scroll_y));
                        let element_idx = (pixel.0 >> 3, pixel.1 >> 3);

                        let element = self.screen_1_elements[element_idx.1 as usize][element_idx.0 as usize];

                        if element.hm {pixel.0 = 7 - pixel.0};
                        if element.vm {pixel.1 = 7 - pixel.1};

                        let raw_px = self.screen_1_tiles[element_idx.1 as usize][element_idx.0 as usize][pixel.1 as usize & 7][pixel.0 as usize & 7];

                        self.color_map[element.palette as usize][raw_px as usize]
                    } else {
                        None
                    }
                {scr1_px} else {
                    if self.color {
                        let mut color = (lcd_ctrl >> 8) & 0x0F;
                        if self.format == PaletteFormat::PLANAR_2BPP {color &= 0x3}
                        let (r, g, b) = self.get_color_palette((lcd_ctrl >> 12) as u8)[color as usize];
                        (r * 17, g * 17, b * 17)
                    } else {
                        let index = ((lcd_ctrl >> 8) & 0x7) as u8;
                        let (port, shift) = (index / 2, index % 2);
                        let color_raw = (self.read_io(0x1C + port as u16) >> shift * 4) & 0x0F;
                        let color = 0xFF - 0x11 * color_raw;

                        (color, color, color)   
                    }
                }
            };

            let dot = (x as usize + y as usize * 224) * 3;

            self.lcd[dot] = pixel.0;
            self.lcd[dot + 1] = pixel.1;
            self.lcd[dot + 2] = pixel.2;
    }

    fn apply_scr2_window(&mut self, s2we: bool, s2wc: bool, x: u8, y: u8) -> Option<(u8, u8, u8)> {
        let scroll_x = self.read_io(0x12);
        let scroll_y = self.read_io(0x13);

        let mut pixel = (x.wrapping_add(scroll_x), y.wrapping_add(scroll_y));
        let element_idx = (pixel.0 >> 3, pixel.1 >> 3);

        let element = self.screen_2_elements[element_idx.1 as usize][element_idx.0 as usize];

        if element.hm {pixel.0 = 7 - pixel.0};
        if element.vm {pixel.1 = 7 - pixel.1};

        let raw_px = self.screen_2_tiles[element_idx.1 as usize][element_idx.0 as usize][pixel.1 as usize & 7][pixel.0 as usize & 7];

        if let Some(color) = self.color_map[element.palette as usize][raw_px as usize] { 
            if s2we {
                let (x1, x2) = (self.read_io(0x08), self.read_io(0x0A));
                if x2 < x1 {return None}
                let (y1, y2) = (self.read_io(0x09), self.read_io(0x0B));
                if y2 < y1 {return None}

                if !(s2wc != (x1..=x2).contains(&x) && s2wc != (y1..=y2).contains(&y)) {
                    return None;
                }
            }
            Some(color)
        } else {
            None
        }
    }

    fn generate_color_map(&mut self) {
        self.color_map = std::array::from_fn(|palette| {
            std::array::from_fn(|raw_px| {
                match self.format {
                    PaletteFormat::PLANAR_2BPP => {
                        if raw_px >= 4 || (raw_px == 0 && palette >= 4) {
                            None
                        } else {
                            Some(if self.color {self.get_color_palette(palette as u8)[raw_px]} else {self.get_monochrome_palette(palette as u8)[raw_px]})
                        }
                    }
                    PaletteFormat::PLANAR_4BPP | PaletteFormat::PACKED_4BPP => {
                        if raw_px == 0 {None} else {Some(self.get_color_palette(palette as u8)[raw_px])}
                    }
                }
            })
        });
    }

    fn get_monochrome_palette(&mut self, palette: u8) -> [(u8, u8, u8); 4] {
        // if palette != 0 {println!("{}", palette)}
        let (lo, hi) = self.read_io_16(0x20 + (palette as u16) * 2);
        let (c0, c1) = (lo & 0x07, (lo >> 4) & 0x07);
        let (c2, c3) = (hi & 0x07, (hi >> 4) & 0x07);

        std::array::from_fn(|i| {
            let raw_px = [c0, c1, c2, c3][i];
            let (port, shift) = (raw_px / 2, raw_px % 2);
            let color_raw = (self.read_io(0x1C + port as u16) >> (shift * 4)) & 0x0F;
            let color = 0xFF - 0x11 * color_raw;

            // if color != 255 {println!("color: {}, raw_px: {}", color, raw_px)};

            (color, color, color)
        })
    }

    fn get_color_palette(&mut self, palette: u8) -> [(u8, u8, u8); 16] {
        let base = 0x0FE00 + (palette as u32) * 32;

        std::array::from_fn(|i| {
            let word = self.read_mem_16(base + i as u32 * 2);
            let (r, g, b) = (((word >> 8) & 0x0F) as u8, ((word >> 4) & 0x0F) as u8, (word & 0x0F) as u8);
            (r * 17, g * 17, b * 17)
        })
    }

    pub fn debug_screen_1(&mut self) {
        let element = self.screen_1_elements[1][11];
        println!("Element: {:#?}", element);
        let base = 0x4000 + (element.tile_idx as u32) * 32;
        println!("Reading tile from {:04X}", base);
        println!("Tile: {:#?}", self.screen_1_tiles[1][11]);
        println!("Correct tile: {:#?}", self.read_tile(element.tile_idx, PaletteFormat::PACKED_4BPP));
        println!("Palette RGB: {:#?}", self.get_color_palette(element.palette));
        println!("Scroll 1 x: {} y: {}", self.read_io(0x10), self.read_io(0x11));
    }

    pub fn debug_screen_2(&mut self) {
        let element = self.screen_2_elements[13][9];
        println!("Element: {:#?}", element);
        let base = 0x4000 + (element.tile_idx as u32) * 32;
        println!("Reading tile from {:04X}", base);
        println!("Tile: {:#?}", self.screen_2_tiles[13][9]);
        println!("Correct tile: {:#?}", self.read_tile(element.tile_idx, PaletteFormat::PACKED_4BPP));
        println!("Palette RGB: {:#?}", self.get_color_palette(element.palette));
        println!("Scroll 1 x: {} y: {}", self.read_io(0x10), self.read_io(0x11));
    }

    pub fn debug_sprites(&mut self) {
        let sprite = self.sprite_table[0];
        println!("Sprite: {:#?}", sprite);
        println!("Sprite base: {:04X}", self.sprite_base);
        println!("SPR_AREA: {:02X}", self.read_io(0x04));
        println!("Sprite tile: {:#?}", self.sprite_tiles[0]);
        println!("Correct tile: {:#?}", self.read_tile(sprite.tile_idx, self.format));
        let lo = self.read_io(0x30 + (sprite.palette as u16) * 2);
        let hi = self.read_io(0x31 + (sprite.palette as u16) * 2);
        let (c0, c1) = (lo & 0x07, (lo >> 4) & 0x07);
        let (c2, c3) = (hi & 0x07, (hi >> 4) & 0x07);
        println!("Palette raw: {:#?}", (c0, c1, c2, c3));
        for i in 0..8 {
            let (port, shift) = (i / 2, i % 2);
            let addr = 0x1C + port;
            let gradation = self.read_io(addr) >> (shift * 4) & 0x0F;
            println!("Gradation {} at port {:02X}, from raw_px {}", gradation, addr, i);
        };
        println!("Palette RGB: {:#?}", self.get_monochrome_palette(sprite.palette));
        // println!("Sprite pixels: {:#?}", self.sprite_pixels);
    }
}