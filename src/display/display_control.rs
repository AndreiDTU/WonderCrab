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

    screen_1_elements: Box<[[ScreenElement; 32]; 32]>,
    screen_2_elements: Box<[[ScreenElement; 32]; 32]>,

    screen_1_tiles: Box<[[[[u8; 8]; 8]; 32]; 32]>,
    screen_2_tiles: Box<[[[[u8; 8]; 8]; 32]; 32]>,

    screen_1_pixels: Box<[[Option<(u8, u8, u8)>; 256]; 256]>,
    screen_2_pixels: Box<[[Option<(u8, u8, u8)>; 256]; 256]>,

    sprites: Box<[SpriteElement; 128]>,
    sprite_tiles: Box<[[[u8; 8]; 8]; 128]>,
    sprite_counter: u8, finished_sprites: bool,

    pub lcd: Box<[(u8, u8, u8); 224 * 144]>,

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
            screen_1_elements: Box::new([[ScreenElement::dummy(); 32]; 32]), screen_2_elements: Box::new([[ScreenElement::dummy(); 32]; 32]),
            screen_1_tiles: Box::new([[[[0; 8]; 8]; 32]; 32]), screen_2_tiles: Box::new([[[[0; 8]; 8]; 32]; 32]),
            screen_1_pixels: Box::new([[None; 256]; 256]), screen_2_pixels: Box::new([[None; 256]; 256]),
            sprites: Box::new([SpriteElement::dummy(); 128]), sprite_tiles: Box::new([[[0; 8]; 8]; 128]), sprite_counter: 0, finished_sprites: false,
            lcd: Box::new([(0, 0, 0); 224 * 144]),
        }
    }

    pub fn tick(&mut self) {
        self.format = self.io_bus.borrow_mut().pallete_format();

        let (x, y) = (self.cycle as usize, self.scanline as usize);

        // if y > 140 {println!("(x,y): {} {}", x, y)}

        if self.cycle % 2 == 1 && self.sprite_counter > 0 {
            self.sprites[self.cycle as usize / 2] = self.read_sprite(self.sprite_base + self.cycle as u16 * 4);
            self.sprite_counter -= 1;
        }

        match self.cycle {
            // Find screen 1's tile and element data
            0 => {
                if self.scanline == 0 {
                    self.get_screen_1_base();
                    self.get_sprite_base();
                    self.get_sprite_counter();
                    // println!("Screen 1 base: {:04X}", self.read_mem_16(self.screen_1_base.into()));
                }
                self.finished_sprites = false;

                let row = y / 8;
                let address = self.screen_1_base | ((row as u16) << 6);
                self.screen_1_elements[row][0] = self.read_screen_element(address);
            }
            1..=63 => {
                let (row, col) = (y / 8, x / 2);
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
                self.screen_2_elements[y / 8][0] = self.read_screen_element(self.screen_2_base);
            }
            66..=129 => {
                let (row, col) = (y / 8, (x - 66) / 2);
                if self.cycle % 2 == 1 {
                    self.screen_2_tiles[row][col] = self.read_tile(self.screen_2_elements[row][col].tile_idx, self.format);
                } else {
                    let address = self.screen_2_base | ((row as u16) << 6) | (col as u16 * 2);
                    self.screen_2_elements[row][col] = self.read_screen_element(address);
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
            self.transform_tiles(0, (144 - 1) / 8);
            self.overlay_pixels(self.cycle, 144 - 1);
        }

        // Display pixels of previous scanline
        if (1..=143).contains(&self.scanline) && self.cycle < 224 {
            if self.cycle % 8 == 0 && self.scanline % 8 == 0 {
                self.transform_tiles(self.cycle / 8, self.scanline / 8);
            }
            self.overlay_pixels(self.cycle, self.scanline - 1);
        }

        if self.scanline == 143 && self.cycle == 225 {
            self.io_bus.borrow_mut().vblank();
        }

        if self.scanline == 159 {
            self.scanline = 0;
            self.io_bus.borrow_mut().set_lcd_line(self.scanline);
        }

        self.cycle = self.cycle.wrapping_add(1);
    }

    fn get_screen_1_base(&mut self) {
        self.screen_1_base = ((self.io_bus.borrow_mut().read_io(0x07) & 0x0F) as u16) << 11;
        if !self.io_bus.borrow_mut().color_mode() {self.screen_1_base &= 0x3800}
        // println!("Screen 1 base: {:014X}", self.screen_1_base);
    }

    fn get_screen_2_base(&mut self) {
        self.screen_2_base = (((self.io_bus.borrow_mut().read_io(0x07) >> 4) & 0x0F) as u16) << 11;
        if !self.io_bus.borrow_mut().color_mode() {self.screen_2_base &= 0x3800}
    }

    fn get_sprite_base(&mut self) {
        self.sprite_base = (self.read_io(0x04) as u16) << 9;
        if !self.io_bus.borrow_mut().color_mode() {self.sprite_base &= 0x3E00}
    }

    fn get_sprite_counter(&mut self) {
        self.sprite_counter = self.read_io(0x06) & 0x7F;
    }

    fn transform_tiles(&mut self, x: u8, y: u8) {
        let tile1 = &mut self.screen_1_tiles[y as usize][x as usize];
        let tile2 = &mut self.screen_2_tiles[y as usize][x as usize];
        
        let element1 = self.screen_1_elements[y as usize][x as usize];
        let element2 = self.screen_2_elements[y as usize][x as usize];

        if element1.vm {tile1.reverse()}
        if element1.hm {
            for row in tile1 {
                row.reverse();
            }
        }

        if element2.vm {tile2.reverse()}
        if element2.hm {
            for row in tile2 {
                row.reverse();
            }
        }
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
                    let data = self.read_mem_32(base + (row as u32 * 2));
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
        let color = self.io_bus.borrow_mut().color_mode();

        let word = self.read_mem_16(addr);

        let vm = word & (1 << 15) != 0;
        let hm = word & (1 << 14) != 0;
        let palette = ((word >> 9) & 0x0F) as u8;
        let mut tile_idx = word & 0x00FF;
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

    fn overlay_pixels(&mut self, x: u8, y: u8) {
        // println!("{}, {}", x, y);
        let (x, y) = (x as usize, y as usize);
        let (lo, hi) = self.io_bus.borrow_mut().read_io_16(0x00);
        let lcd_ctrl = u16::from_le_bytes([lo, hi]);

        let scr1  = lcd_ctrl & 1 != 0;
        let scr2  = (lcd_ctrl >> 1) & 1 != 0;
        let spr   = (lcd_ctrl >> 2) & 1 != 0;
        let sprwe = (lcd_ctrl >> 3) & 1 != 0;
        let s2we  = (lcd_ctrl >> 4) & 1 != 0;
        let s2wc  = (lcd_ctrl >> 5) & 1 != 0;

        let bg_color = if self.io_bus.borrow_mut().color_mode() {
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
        };

        if scr1 {
            let (scroll_1_x, scroll_1_y) = self.io_bus.borrow_mut().read_io_16(0x10);
            let (scroll_1_x, scroll_1_y) = (scroll_1_x as usize, scroll_1_y as usize);

            let (y1, x1) = (y.wrapping_add(scroll_1_y), x.wrapping_add(scroll_1_x));

            let palette = self.screen_1_elements[y1 / 8][x1 / 8].palette;
            let raw_px = self.screen_1_tiles[y1 / 8][x1 / 8][y1 % 8][x1 % 8];

            self.screen_1_pixels[y][x] = self.fetch_pixel_color(palette, raw_px);
            // if true || (self.screen_1_pixels[y][x] != None && self.screen_1_pixels[y][x] != Some((0, 0, 0))) {println!("screen_1_pixels[{y}][{x}] set to {:?}", self.screen_1_pixels[y][x]);}
        } else {
            self.screen_2_pixels[y][x] = None;
        }

        if scr2 {
            self.screen_2_pixels[y][x] = self.apply_scr2_window(s2we, s2wc, x as u8, y as u8);
            // if true || (self.screen_2_pixels[y][x] != None && self.screen_2_pixels[y][x] != Some((0, 0, 0))) {println!("screen_2_pixels[{y}][{x}] set to {:?}", self.screen_2_pixels[y][x]);}
        } else {
            self.screen_2_pixels[y][x] = None;
        }

        self.lcd[x + y * 224] = 
            if let Some(scr2_px) = self.screen_2_pixels[y][x] {scr2_px}
            else if let Some(scr1_px) = self.screen_1_pixels[y][x] {scr1_px}
            else {bg_color}
    }

    fn apply_scr2_window(&mut self, s2we: bool, s2wc: bool, x: u8, y: u8) -> Option<(u8, u8, u8)> {
        if s2we {
            let (x1, x2) = (self.read_io(0x08), self.read_io(0x0A));
            if x2 < x1 {return None}
            let (y1, y2) = (self.read_io(0x09), self.read_io(0x0B));
            if y2 < y1 {return None}

            if !(s2wc == (x1..=x2).contains(&x) && s2wc == (y1..=y2).contains(&y)) {
                return None;
            }
        }

        let (scroll_2_x, scroll_2_y) = self.io_bus.borrow_mut().read_io_16(0x12);

        let (y2, x2) = (self.scanline.wrapping_add(scroll_2_y) as usize, self.cycle.wrapping_add(scroll_2_x) as usize);

        // if y2 >= 8 || x2 >= 8 {println!("x: {} y: {}", x2, y2)}

        let palette = self.screen_2_elements[y2 / 8][x2 / 8].palette;
        let raw_px = self.screen_2_tiles[y2 / 8][x2 / 8][y2 % 8][x2 % 8];

        self.fetch_pixel_color(palette, raw_px)
    }

    fn fetch_pixel_color(&mut self, palette: u8, raw_px: u8) -> Option<(u8, u8, u8)> {
        // if palette != 0 {println!("{}", palette)}
        match self.format {
            PaletteFormat::PLANAR_2BPP => {
                if palette >= 4 && raw_px == 0 {
                    return None;
                }
                // if palette != 0 {println!("{}", palette)}

                Some(if self.io_bus.borrow_mut().color_mode() {
                    let (r, g, b) = self.get_color_palette(palette)[raw_px as usize];
                    (r * 17, g * 17, b * 17)
                } else {
                    self.get_monochrome_palette(palette)[raw_px as usize]
                })
            },
            PaletteFormat::PLANAR_4BPP | PaletteFormat::PACKED_4BPP => {
                if raw_px == 0 {
                    return None;
                }
                let (r, g, b) = self.get_color_palette(palette)[raw_px as usize];

                Some((r * 17, g * 17, b * 17))
            },
        }
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

    fn get_color_palette(&mut self, index: u8) -> [(u8, u8, u8); 16] {
        let base = 0x0FE00 + (index as u32) * 32;

        std::array::from_fn(|i| {
            let word = self.read_mem_16(base + i as u32 * 2);
            ((word & 0x0F) as u8, ((word >> 4) & 0x0F) as u8, ((word >> 8) & 0x0F) as u8)
        })
    }

    pub fn fetch_map_word(&self, bank_nibble: u8, row: usize, col: usize) -> u16 {
        let bank  = (bank_nibble & 0x0F) as u32;
        let base  = bank << 11;
        let idx   = (row * 32 + col) as u32;
        self.mem_bus.borrow_mut().read_mem_16(base + idx*2)
    }

    pub fn debug_screen_1(&mut self) {
        let element = self.screen_1_elements[0][0];
        println!("Element: {:#?}", element);
        let base = 0x2000 + (element.tile_idx as u32) * 16;
        println!("Reading tile from {:04X}", base);
        println!("Tile: {:#?}", self.screen_1_tiles[0][0]);
        println!("Correct tile: {:#?}", self.read_tile(element.tile_idx, PaletteFormat::PLANAR_2BPP));
        let lo = self.read_io(0x20 + (self.screen_1_elements[0][0].palette as u16) * 2);
        let hi = self.read_io(0x21 + (self.screen_1_elements[0][0].palette as u16) * 2);
        let (c0, c1) = (lo & 0x07, (lo >> 4) & 0x07);
        let (c2, c3) = (hi & 0x07, (hi >> 4) & 0x07);
        println!("Palette raw: {:#?}", (c0, c1, c2, c3));
        for i in 0..8 {
            let (port, shift) = (i / 2, i % 2);
            let addr = 0x1C + port;
            let gradation = self.read_io(addr) >> (shift * 4) & 0x0F;
            println!("Gradation {} at port {:02X}, from raw_px {}", gradation, addr, i);
        };
        println!("Port 0x1F: {:02X}", self.read_io(0x1F));
        println!("Palette RGB: {:#?}", self.get_monochrome_palette(self.screen_1_elements[0][0].palette));
        println!("{:#?}", self.read_io_16(0x20));
    }

    #[cfg(test)]
    pub fn decode_2bpp_tile(&mut self, index: u16) -> [[u8; 8]; 8] {
        self.read_tile(index, PaletteFormat::PLANAR_2BPP)
    }

    #[cfg(test)]
    pub fn force_get_monochrome_palette(&mut self, index: u8) -> [(u8, u8, u8); 4] {
        self.get_monochrome_palette(index)
    }
}

#[cfg(test)]
mod test {
    use crate::bus::io_bus::IOBusConnection;
    use crate::bus::mem_bus::MemBusConnection;
    use crate::display::PaletteFormat;
    use crate::SoC;

    #[test]
    fn test_overlay_single_pixel() {
        let mut soc = SoC::test_build();

        // 1) Enable screen1 in DISPLAY_CTRL
        soc.write_io_16(0x00, 0x0001);

        // 2) Set SCR_AREA so screen1 base = 0x0000 (bank 0)
        soc.write_io(0x07, 0x00);

        // 3) Write a single map tile at [0,0]:
        //    no flips, bank=0, palette=1, idx=0
        let map_word = (1u16 << 9) | 0;
        soc.write_mem_16(0 * 2, map_word);

        // 4) Write tile #0 row 0 such that pixel (0,0)=1:
        //    plane0=0x80, plane1=0x00
        let tile_base = 0x2000 + 0 * 16;
        soc.write_mem(tile_base + 0*2,     0x80);
        soc.write_mem(tile_base + 0*2 + 1, 0x00);

        // 5) Seed palette1 so entry1 = mid‐gray:
        //    nibble registers:
        soc.write_io(0x22, 0x10); // c1=1
        soc.write_io(0x23, 0x10); // c3=1 for both halves
        //    color RAM:
        soc.write_io(0x1C, 0x10);
        soc.write_io(0x1D, 0x10);

        // 6) Manually populate the display’s element/tile caches:
        soc.get_display().get_screen_1_base();
        soc.get_display().screen_1_elements[0][0] = soc.get_display().read_screen_element(0);
        soc.get_display().screen_1_tiles   [0][0] = soc.get_display().read_tile(0, PaletteFormat::PLANAR_2BPP);

        // 7) Call overlay for pixel (0,0):
        soc.get_display().overlay_pixels(0, 0);

        // Expect lcd[0] to be palette1.entry1 = (0xEE,0xEE,0xEE)
        assert_eq!(soc.get_display().lcd[0], (0xEE, 0xEE, 0xEE));
    }
}