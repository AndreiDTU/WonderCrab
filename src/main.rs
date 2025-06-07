use std::{collections::HashMap, env, sync::{Arc, Mutex}, time::{Duration, Instant}};

use cartridge::Mapper;
use keypad::Keys;
use mimalloc::MiMalloc;
use sdl2::{audio::{AudioCallback, AudioQueue, AudioSpecDesired}, event::Event, keyboard::Keycode, pixels::PixelFormatEnum, rect::Rect};
use soc::SoC;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub mod soc;
pub mod bus;
pub mod dma;
pub mod keypad;
pub mod sound;

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod display;

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod cartridge;

#[allow(non_snake_case)]
pub mod cpu;

const WINDOW_WIDTH: u32 = 1344;
const WINDOW_HEIGHT: u32 = 864;

const FRAME_WIDTH: u32 = 224;
const FRAME_HEIGHT: u32 = 144;

struct SampleStream {
    samples: Arc<Mutex<Vec<(u16, u16)>>>
}

impl AudioCallback for SampleStream {
    type Channel = u8;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        let mut buffer = self.samples.lock().unwrap();
        for request in out {
            if let Some(sample) = buffer.pop() {
                *request = sample.0 as u8
            }
        }
    }
}

fn main() -> Result<(), String> {
    let args: Vec<_> = env::args().collect();
    let game = if args.len() > 1 {Some(&args[1])} else {None};
    let trace = args.get(2) == Some(&"trace".to_string());
    let mute = args.get(2) == Some(&"mute".to_string()) || trace;

    let samples = Arc::new(Mutex::new(Vec::new()));

    let mut soc = if let Some(game) = game {
        let (color, ram_content, rom, mapper, sram) = parse_rom(game);
        SoC::new(color, ram_content, rom, mapper, sram, trace, Arc::clone(&samples), mute)
    } else {SoC::test_build()};

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("WonderSwan", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build().unwrap();

    let audio_subsystem = sdl_context.audio()?;
    let desired_spec = AudioSpecDesired {
        freq: Some(24000),
        channels: Some(1),
        samples: Some(1024),
    };
    let audio_device = audio_subsystem.open_playback(None, &desired_spec, |_| SampleStream {samples: Arc::clone(&samples)})?;
    audio_device.resume();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_logical_size(FRAME_WIDTH, FRAME_HEIGHT).unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, FRAME_WIDTH, FRAME_HEIGHT).unwrap();
    let mut event_pump = sdl_context.event_pump()?;

    let mut key_map = HashMap::new();
    key_map.insert(Keycode::A, Keys::Y1);
    key_map.insert(Keycode::W, Keys::Y2);
    key_map.insert(Keycode::D, Keys::Y3);
    key_map.insert(Keycode::S, Keys::Y4);
    key_map.insert(Keycode::U, Keys::X1);
    key_map.insert(Keycode::K, Keys::X2);
    key_map.insert(Keycode::J, Keys::X3);
    key_map.insert(Keycode::H, Keys::X4);
    key_map.insert(Keycode::KP_4, Keys::X1);
    key_map.insert(Keycode::KP_8, Keys::X2);
    key_map.insert(Keycode::KP_6, Keys::X3);
    key_map.insert(Keycode::KP_5, Keys::X4);
    key_map.insert(Keycode::Return, Keys::Start);
    key_map.insert(Keycode::Z, Keys::B);
    key_map.insert(Keycode::X, Keys::A);

    let mut previous = Instant::now();
    let mut rotated = false;
    let mut dst = Rect::new(0, 0, FRAME_WIDTH, FRAME_HEIGHT);
    let mut first_frame = true;

    loop {
        if soc.tick() {
            let now = Instant::now();
            let delta = if first_frame {
                first_frame = false;
                Instant::now() - Instant::now()
            } else {
                now - previous
            };
            previous = now;

            std::thread::sleep(Duration::from_micros(13_250u64.saturating_sub(delta.as_micros() as u64)));

            canvas.clear();

            let frame = soc.get_lcd();
            texture.update(None,&frame.borrow()[..], FRAME_WIDTH as usize * 3).unwrap();
            
            let angle = if rotated {270.0} else {0.0};
            if rotated {
                canvas.copy_ex(&texture, None, dst, angle, None, false, false).unwrap();
            } else {
                canvas.copy(&texture, None, None)?;
            }
            canvas.present();
            
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                        // for addr in 0x3B52..=0x3B53 {println!("SCREEN ELEMENT: [{:04X}] = {:02X}", addr, soc.read_mem(addr))}
                        // for addr in 0x4340..=0x435F {println!("TILE: [{:04X}] = {:02X}", addr, soc.read_mem(addr))}
                        // soc.get_display().debug_screen_2();
                        // soc.io_bus.borrow().debug_eeprom();
                        return Ok(());
                    },
                    Event::KeyDown { keycode, .. } => {
                        if let Some(key) = keycode {
                            if let Some(Keycode::R) = keycode {
                                rotated = !rotated;
                                if rotated {
                                    canvas.window_mut().set_size(WINDOW_HEIGHT, WINDOW_WIDTH).unwrap();
                                    canvas.window_mut().set_position(sdl2::video::WindowPos::Centered, sdl2::video::WindowPos::Centered);
                                    canvas.set_logical_size(FRAME_HEIGHT, FRAME_WIDTH).unwrap();
                                    dst.set_x(-40);
                                    dst.set_y(40);
                                    canvas.clear();
                                } else {
                                    canvas.window_mut().set_size(WINDOW_WIDTH, WINDOW_HEIGHT).unwrap();
                                    canvas.window_mut().set_position(sdl2::video::WindowPos::Centered, sdl2::video::WindowPos::Centered);
                                    canvas.set_logical_size(FRAME_WIDTH, FRAME_HEIGHT).unwrap();
                                    dst.set_x(0);
                                    dst.set_y(0);
                                    canvas.clear();
                                }
                            }
                            if let Some(key) = key_map.get(&key) {
                                soc.io_bus.borrow_mut().keypad.borrow_mut().set_key(*key, true);
                            }
                        }
                    }
                    Event::KeyUp { keycode, .. } => {
                        if let Some(key) = keycode {
                            if let Some(key) = key_map.get(&key) {
                                soc.io_bus.borrow_mut().keypad.borrow_mut().set_key(*key, false);
                            }
                        }
                    }
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
    let (ram_size, sram) = match footer[0xB] {
        0x00 => (0x0u32, true),
        0x01 | 0x02 => (0x08000, true),
        0x03 => (0x20000, true),
        0x04 => (0x40000, true),
        0x05 => (0x80000, true),
        0x10 => (0x0400, false),
        0x20 => (0x4000, false),
        0x50 => (0x2000, false),
        _ => panic!("Unknown save type!")
    };
    let save = std::fs::read(format!("{}.sav", game)).or_else(|_| {Ok::<_, ()>(vec![0; ram_size as usize])}).unwrap();

    let mapper = match footer[0xD] {
        0 => Mapper::B_2001,
        1 => Mapper::B_2003,
        _ => panic!("Unknown mapper!"),
    };

    // if mapper == Mapper::B_2003 {println!("Mapper 2003")}

    (color, save, rom, mapper, sram)
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
