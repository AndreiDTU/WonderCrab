use std::{cell::RefCell, rc::Rc};

use bitflags::bitflags;

use crate::{bus::{io_bus::{IOBus, IOBusConnection}, mem_bus::{MemBus, MemBusConnection}}, sound::channel::Channel};

mod channel;

bitflags! {
    #[derive(Clone, Copy)]
    pub struct SoundControl: u8 {
        const NOISE = 0b1000_0000;
        const SWEEP = 0b0100_0000;
        const VOICE = 0b0010_0000;
        
        const Enb4 = 0b0000_1000;
        const Enb3 = 0b0000_0100;
        const Enb2 = 0b0000_0010;
        const Enb1 = 0b0000_0001;
    }
}

pub struct Sound {
    mem_bus: Rc<RefCell<MemBus>>,
    io_bus: Rc<RefCell<IOBus>>,

    channel_1: Channel,
    channel_2: Channel,
    channel_3: Channel,
    channel_4: Channel,

    control: SoundControl,

    sweep_clock: usize,
    step_clock: usize,

    noise_clock: u16,
    noise: Option<u8>,
}

impl Sound {
    pub fn new(mem_bus: Rc<RefCell<MemBus>>, io_bus: Rc<RefCell<IOBus>>) -> Self {
        let [channel_1, channel_2, channel_3, channel_4] = [Channel::new(); 4];
        Self {
            mem_bus, io_bus,
            
            channel_1, channel_2, channel_3, channel_4,

            control: SoundControl::from_bits_truncate(0),

            sweep_clock: 0, step_clock: 0,
            noise_clock: 0, noise: None,
        }
    }

    pub fn tick(&mut self) -> (u16, u16) {
        self.control = SoundControl::from_bits_truncate(self.read_io(0x90));
        self.sweep();
        self.noise();
        self.load_waveforms();
        self.load_frequencies();

        let samples = self.channel_outputs();

        let volumes: [(u8, u8); 4] = std::array::from_fn(|i| {
            let volume = self.read_io(0x88 + i as u16);
            (volume >> 4, volume & 0xF)
        });

        let mut stereo_samples: [(u8, u8); 4] = std::array::from_fn(|i| {
            (samples[i] * volumes[i].0, samples[i] * volumes[i].1)
        });

        if self.control.contains(SoundControl::VOICE) {
            let voice = samples[1];
            let voice_volume = self.read_io(0x94);

            let right = if voice_volume & 0b0001 != 0 {
                voice
            } else if voice_volume & 0b0010 != 0 {
                voice >> 1
            } else {0};

            let left = if voice_volume & 0b0100 != 0 {
                voice
            } else if voice_volume & 0b1000 != 0 {
                voice >> 1
            } else {0};

            stereo_samples[1] = (left, right);
        }

        let stereo_output = stereo_samples.iter()
            .copied()
            .map(|(l, r)| (l as u16, r as u16))
            .reduce(|(left_out, right_out), (left_in, right_in)| (left_out + left_in, right_out + right_in))
            .unwrap();

        let out_ctrl = self.read_io(0x91);
        if out_ctrl & 0x80 != 0 {
            panic!("Headphones not yey implemented!");
        } else {
            let rng_s = (out_ctrl >> 1) & 3;
            let output = ((stereo_output.0 + stereo_output.1) >> rng_s) as u8;
            (output as u16, output as u16)
        }
    }

    fn channel_outputs(&mut self) -> [u8; 4] {
        let sample_2 = if self.control.contains(SoundControl::Enb2) {
            self.channel_2.tick()
        } else {0};

        let sample_4 = if self.control.contains(SoundControl::Enb4) {
            self.channel_4.tick()
        } else {0};

        [
            if self.control.contains(SoundControl::Enb1) {
                self.channel_1.tick()
            } else {0},

            if self.control.contains(SoundControl::VOICE) {
                self.read_io(0x89)
            } else {sample_2},

            if self.control.contains(SoundControl::Enb3) {
                self.channel_3.tick()
            } else {0},

            if let Some(noise) = self.noise {
                noise
            } else {sample_4},
        ]
    }

    fn load_waveforms(&mut self) {
        let wave_p = self.read_io(0x8F) as u32;
        let base = wave_p << 6;

        let groups: [[u8; 16]; 4] = std::array::from_fn(|channel| {
            std::array::from_fn(|index| {
                let addr = base + (index as u32) + ((channel * 16) as u32);
                self.read_mem(addr)
            })
        });

        self.channel_1.waveform = groups[0];
        self.channel_2.waveform = groups[1];
        self.channel_3.waveform = groups[2];
        self.channel_4.waveform = groups[3];
    }

    fn load_frequencies(&mut self) {
        let frequencies: [u16; 4] = std::array::from_fn(|i| {
            let (lo, hi) = self.read_io_16(0x80 + (i * 2) as u16);
            u16::from_le_bytes([lo, hi]) & 0x7FF
        });

        self.channel_1.frequency = 2048 - frequencies[0];
        self.channel_2.frequency = 2048 - frequencies[1];
        self.channel_3.frequency = 2048 - frequencies[2];
        self.channel_4.frequency = 2048 - frequencies[3];
    }

    fn sweep(&mut self) {
        if self.control.contains(SoundControl::SWEEP) && self.control.contains(SoundControl::Enb3) {
            self.sweep_clock += 1;
            if self.sweep_clock > 8192 {
                self.sweep_clock = 0;
                if self.step_clock == 0 {
                    self.step_clock = ((self.read_io(0x8D) & 0x1F) - 1) as usize;
                    let sweep = self.read_io(0x8C) as i8 as i16;
                    let (lo, hi) = self.read_io_16(0x84);
                    let old_frequency = (u16::from_le_bytes([lo, hi]) & 0x7FF) as i16;
                    let mut new_frequency = old_frequency + sweep;
                    if new_frequency > 2047 {
                        new_frequency = 0;
                    } else if new_frequency < 0 {
                        new_frequency = 2047;
                    }
                    self.write_io_16(0x84, (new_frequency as u16) & 0x7FF);
                } else {
                    self.step_clock -= 1;
                }
            }
        }
    }

    fn noise(&mut self) {
        if self.control.contains(SoundControl::NOISE) && self.control.contains(SoundControl::Enb4) {
            let noise_ctrl = self.read_io(0x8E);
            if noise_ctrl & 0x10 == 0 {return}

            if self.noise_clock == 0 {
                let (lo, hi) = self.read_io_16(0x86);
                self.noise_clock = 2048 - u16::from_le_bytes([lo, hi]) & 0x1FF;

                if noise_ctrl & 0x08 != 0 {
                    self.write_io(0x92, 0);
                    self.write_io(0x8E, noise_ctrl & 0xF7);
                }

                let (lo, hi) = self.read_io_16(0x92);
                let mut lsfr = u16::from_le_bytes([lo, hi]) & 0x7FFF;

                let tap = noise_ctrl & 7;
                let tap_bit = (lsfr >> match tap {
                    0 => 14,
                    1 => 10,
                    2 => 13,
                    3 => 4,
                    4 => 8,
                    5 => 6,
                    6 => 9,
                    7 => 11,
                    _ => unreachable!(),
                }) & 1;

                let random_bit = (lsfr >> 7) ^ tap_bit != 0;
                lsfr <<= 1;
                lsfr &= 0x7FFF;
                lsfr |= random_bit as u16;
                self.io_bus.borrow_mut().set_lsfr(lsfr);
                self.noise = Some(if random_bit {0xFF} else {0x00});
            } else {
                self.noise_clock -= 1;
            }
        } else {
            self.noise = None;
        }
    }
}

impl MemBusConnection for Sound {
    fn read_mem(&mut self, addr: u32) -> u8 {
        self.mem_bus.borrow_mut().read_mem(addr)
    }

    fn write_mem(&mut self, addr: u32, byte: u8) {
        self.mem_bus.borrow_mut().write_mem(addr, byte);
    }
}

impl IOBusConnection for Sound {
    fn read_io(&mut self, addr: u16) -> u8 {
        self.io_bus.borrow_mut().read_io(addr)
    }

    fn write_io(&mut self, addr: u16, byte: u8) {
        self.io_bus.borrow_mut().write_io(addr, byte);
    }
}