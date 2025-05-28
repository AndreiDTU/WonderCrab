use std::env;

use cartridge::Mapper;
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};
use soc::SoC;

pub mod soc;
pub mod bus;
pub mod dma;

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod display;

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod cartridge;

#[allow(non_snake_case)]
pub mod cpu;

fn main() -> Result<(), String> {
    let args: Vec<_> = env::args().collect();
    let game = if args.len() > 1 {Some(&args[1])} else {None};

    let mut soc = if let Some(game) = game {
        let (color, sram, rom, mapper, rewrittable) = parse_rom(game);
        SoC::new(color, sram, rom, mapper, rewrittable)
    } else {SoC::test_build()};

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("WonderSwan", 1792, 1152)
        .position_centered()
        .build().unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, 224, 144).unwrap();
    let mut event_pump = sdl_context.event_pump()?;

    loop {
        if soc.tick() {
            let mut frame = Vec::with_capacity(soc.get_lcd().borrow().len() * 3);
            for &(r, g, b) in soc.get_lcd().borrow().iter() {
                frame.push(r);
                frame.push(g);
                frame.push(b);
}
            texture.update(None,&frame, 224*3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => std::process::exit(0),
                    _ => {}
                }
            }
        }
    }
}

fn parse_rom(game: &str) -> (bool, Vec<u8>, Vec<u8>, Mapper, bool) {
    let rom = std::fs::read(format!("{}.ws", game)).or_else(|_| {std::fs::read(format!("{}.wsc", game))}).unwrap();
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
