//! Basic WonderSwan emulator
//! 
//! I made this as a learning project and to have something to put on my resume
//! It might be useful as a reference for similar projects or as a basis for a more accurate emulator
//! 
//! Use something like Mesen or Ares if you actually want to play games though

#[warn(missing_docs)]

use std::{cell::RefCell, collections::HashMap, env, rc::Rc, sync::{Arc, Mutex}, time::{Duration, Instant}};

use cartridge::Mapper;
use mimalloc::MiMalloc;
use sdl2::{audio::{AudioCallback, AudioSpecDesired}, event::Event, keyboard::Keycode, pixels::PixelFormatEnum, rect::Rect};
use soc::SoC;

use crate::bus::io_bus::{keypad::Keys, IOBus};

#[global_allocator]
/// This is a fast memory allocator made by Microsoft.
/// 
/// It improved performance quite significantly when I added it.
static GLOBAL: MiMalloc = MiMalloc;

/// This module contains the I/O and memory busses
/// 
/// The WonderSwan contains only a single memory bus and a single I/O bus.
/// These classes are therefore intended to produce singletons, to which multiple
/// references can be shared between the different components, mimicking the
/// system's original architecture.
pub mod bus;

/// This module contains the cartridge
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod cartridge;

/// This module contains the WonderSwan's CPU
/// 
/// This file's contents specifically are made up of things that would be useful to both defining the opcodes and operating the CPU
#[allow(non_snake_case)]
pub mod cpu;

/// This module contains the WonderSwan's display chip
/// 
/// Actually displaying the screen to the Window is hadnled through SDL in main
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod display;

/// The WonderSwan color and WonderCrystal DMAs
pub mod dma;

/// System on a chip
pub mod soc;

/// The WonderSwan's sound chip
pub mod sound;

/// Width of the window that appears when you run the program
const WINDOW_WIDTH: u32 = 1344;
/// Height of the window that appears when you run the program
const WINDOW_HEIGHT: u32 = 864;

/// Width of the WonderSwan's screen when in landscape orientation
const FRAME_WIDTH: u32 = 224;
/// Height of the WonderSwan's screen when in landscape orientation
const FRAME_HEIGHT: u32 = 144;

/// A struct holding a vector of audio samples behind a Mutex
/// 
/// The samples in here are generated by the audio system and the vector is updated at the WonderSwan's samplerate of 24kHz
struct SampleStream {
    /// Vector containing the samples
    /// 
    /// In the current implementation only the 8-bit monaural speaker audio is supported.
    /// The vector is set up to contain u16 tuplets to make it easier to extend this project
    /// to output stereo 16-bit headphone audio.
    samples: Arc<Mutex<Vec<(u16, u16)>>>
}

/// This block will likely need to be rewritten to add headphone support.
/// 
/// It currently outputs only the low byte of the left stereo channel.
/// This is not a problem for the current implementation as only monaural audio is supported.
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

/// The emulator's main function
/// 
/// It is mainly concerned with SDL features.
/// 
/// # Panics
/// 
/// This will panic when any of the SDL functions called return an `Err<T>` where T is not String.
/// If an `Err<String>` is produced it will instead return it and close the emulator.
fn main() -> Result<(), String> {
    let args: Vec<_> = env::args().collect();
    let game = if args.len() > 1 {Some(&args[1])} else {None};
    let trace = args.get(2) == Some(&"trace".to_string());
    let mute = args.get(2) == Some(&"mute".to_string()) || trace;

    let samples = Arc::new(Mutex::new(Vec::new()));

    let mut global_color = false;

    let mut soc = if let Some(game) = game {
        let (color, ram_content, ieeprom, eeprom, rom, mapper, sram, rom_info) = parse_rom(game);
        global_color = color;
        SoC::new(color, ram_content, ieeprom, eeprom, rom, mapper, sram, trace, Arc::clone(&samples), mute, rom_info)
    } else {SoC::test_build()};

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("WonderCrab", WINDOW_WIDTH, WINDOW_HEIGHT)
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
                        // soc.get_display().debug_screen_1();
                        // soc.io_bus.borrow().debug_eeprom();
                        if let Some(game) = game {save_game(soc.io_bus, global_color, game)};
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
                            // Tracing makes the framerate unplayable,
                            // this is disabled to make sure the user
                            // doesn't press it by accident
                            
                            /*
                            if let Some(Keycode::T) = keycode {
                                soc.cpu.trace = !trace;
                                soc.mute = !mute;
                            }
                            */
                            
                            if let Some(key) = key_map.get(&key) {
                                soc.io_bus.borrow_mut().set_key(*key, true);
                            }
                        }
                    }
                    Event::KeyUp { keycode, .. } => {
                        if let Some(key) = keycode {
                            if let Some(key) = key_map.get(&key) {
                                soc.io_bus.borrow_mut().set_key(*key, false);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Extracts information from the requested ROM image and any existing save files
/// 
/// # Return value
/// This function returns a tuple containing the following:
/// - `color: bool` whether or not the ROM supports color output
/// - `save: Vec<u8>` contents of SRAM
/// - `ieeprom: Vec<u8>` contents of the IEEPROM
/// - `eeprom: Vec<u8>` contents of the cartridge EEPROM
/// - `rom: Vec<u8>` contents of the ROM
/// - `mapper: Mapper` the mapper chip used by the cartridge
/// - `sram: bool` whether or not the cartridge contains SRAM
/// - `rom_info: u8` bits 2 and 3 of the system control port 0xA0
fn parse_rom(game: &str) -> (bool, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Mapper, bool, u8) {
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

    let ieeprom_path = if color {"wsc.ieeprom"} else {"ws.ieeprom"};
    let eeprom_path = format!("{}.eeprom", game);
    let sram_path = format!("{}.sram", game);

    let ieeprom = std::fs::read(ieeprom_path).or_else(|_| Ok::<_, ()>(Vec::new())).unwrap();
    let eeprom = std::fs::read(eeprom_path).or_else(|_| Ok::<_, ()>(Vec::new())).unwrap();
    let save = std::fs::read(sram_path).or_else(|_| {Ok::<_, ()>(vec![0; ram_size as usize])}).unwrap();

    let mapper = match footer[0xD] {
        0 => Mapper::B_2001,
        1 => Mapper::B_2003,
        _ => panic!("Unknown mapper!"),
    };

    let rom_info = footer[0xC] & 0x0C;

    if mapper == Mapper::B_2003 {println!("Mapper 2003")}

    (color, save, ieeprom, eeprom, rom, mapper, sram, rom_info)
}

/// Saves the game and console's rewrittable memory to files
/// 
/// This function will save the contents of the following media to the following addresses:
/// 
/// - IEEPROM to either wsc.ieeprom or ws.ieeprom depending on color
/// - Cart EEPROM to \[game\].eeprom
/// - SRAM to \[game\].sram
fn save_game(io_bus: Rc<RefCell<IOBus>>, color: bool, game: &str) {
    let local_io_bus = io_bus.borrow();
    let ieeprom = &local_io_bus.ieeprom;
    let eeprom = &local_io_bus.eeprom;
    let sram = &local_io_bus.cartridge.borrow().sram;

    let ieeprom_path = if color {"wsc.ieeprom"} else {"ws.ieeprom"};
    let eeprom_path = format!("{}.eeprom", game);
    let sram_path = format!("{}.sram", game);

    std::fs::write(ieeprom_path, ieeprom.contents.clone()).unwrap();
    if let Some(eeprom) = eeprom {std::fs::write(eeprom_path, eeprom.contents.clone()).unwrap()}
    if !sram.is_empty() {std::fs::write(sram_path, sram.clone()).unwrap()}
}

/// Same as assert_eq but prints the values in hex instead
/// 
/// I wrote it so it so it would be easier to make CPU tests
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
