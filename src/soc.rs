use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}};

use crate::{bus::{io_bus::{IOBus, IOBusConnection}, mem_bus::{MemBus, MemBusConnection, Owner}}, cartridge::{Cartridge, Mapper}, cpu::v30mz::V30MZ, display::display_control::Display, dma::{gdma::GDMA, sdma::SDMA, DMA}, sound::Sound};

/// System on a chip
/// 
/// The SoC decides which component is ticked and when, it also handles output to main.
pub struct SoC {
    /// CPU, an NEC V30MZ
    pub(super) cpu: V30MZ,
    /// General DMA
    gdma: GDMA,
    /// Sound DMA
    sdma: SDMA,
    /// Sound chip
    sound: Sound,
    /// Display chip
    display: Display,

    /// A reference to the shared memory bus
    mem_bus: Rc<RefCell<MemBus>>,
    /// A reference to the shared I/O bus
    pub(super) io_bus: Rc<RefCell<IOBus>>,

    /// The master clock cycle divided by 4 and reset on each new frame
    cycles: usize,

    /// The vector shared with the audio thread
    pub(super) samples: Arc<Mutex<Vec<(u16, u16)>>>,
    /// A counter for how many cycles there have been since the last sample was pushed
    sample_acc: u64,
    /// A counter for how many cycles have been pushed since the SDMA last operated
    sdma_clock: u8,

    /// The LCD shared with the display chip and SDL
    lcd: Rc<RefCell<[u8; 3 * 224 * 144]>>,

    /// Mute flag, if set will stop the SoC from pushing samples
    pub(super) mute: bool,
}

impl MemBusConnection for SoC {
    fn read_mem(&mut self, addr: u32) -> u8 {
        self.mem_bus.borrow_mut().read_mem(addr)
    } 

    fn write_mem(&mut self, addr: u32, byte: u8) {
        self.mem_bus.borrow_mut().write_mem(addr, byte);
    }
}

impl IOBusConnection for SoC {
    fn read_io(&mut self, addr: u16) -> u8 {
        self.io_bus.borrow_mut().read_io(addr)
    }
    
    fn write_io(&mut self, addr: u16, byte: u8) {
        self.io_bus.borrow_mut().write_io(addr, byte);
    }
}

impl SoC {
    /// Generates a new SoC
    /// 
    /// Requires data about the current ROM, CLI parameters, IEEPROM and a reference to the sample vector
    pub fn new(color: bool, ram_content: Vec<u8>, ieeprom: Vec<u8>, eeprom: Vec<u8>, rom: Vec<u8>, mapper: Mapper, sram: bool, trace: bool, samples: Arc<Mutex<Vec<(u16, u16)>>>, mute: bool, rom_info: u8) -> Self {
        let (cartridge, eeprom) = if sram {
            (Rc::new(RefCell::new(Cartridge::new(mapper, ram_content, rom, sram))), None)
        } else {
            (Rc::new(RefCell::new(Cartridge::new(mapper, Vec::new(), rom, false))), if eeprom.len() > 0 {Some(eeprom)} else {Some(ram_content)})
        };
        let io_bus = Rc::new(RefCell::new(IOBus::new(Rc::clone(&cartridge), ieeprom, eeprom, color, rom_info)));
        let mem_bus = Rc::new(RefCell::new(MemBus::new(Rc::clone(&io_bus), Rc::clone(&cartridge))));
        let mut cpu = V30MZ::new(Rc::clone(&mem_bus), Rc::clone(&io_bus), trace);
        let gdma = GDMA::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let sdma = SDMA::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let sound = Sound::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let lcd = Rc::new(RefCell::new([0; 3 * 224 * 144]));
        let display = Display::new(Rc::clone(&mem_bus), Rc::clone(&io_bus), Rc::clone(&lcd));

        cpu.reset();

        Self {cpu, gdma, sdma, sound, display, mem_bus, io_bus, cycles: 0, samples, sample_acc: 0, sdma_clock: 0, lcd, mute}
    }

    /// Executes four ticks of the master clock, returns true if a new frame has finished rendering
    pub fn tick(&mut self) -> bool {
        if self.gdma.cycles == 0 {
            if self.gdma.is_enabled() {
                self.gdma.start_op();
            }
        }

        if self.gdma.cycles > 0 {
            self.gdma.tick();
        } else {
            if self.sdma.cycles > 0 {
                self.sdma.tick();
            } else {
                self.cpu.tick();
            }
        };

        if self.mem_bus.borrow().owner == Owner::CPU {
            return false;
        }

        let sample = self.sound.tick();
        self.sample_acc += 1;
        if self.sample_acc >= 128 {
            self.sample_acc -= 128;
            self.sdma_clock += 1;
            if self.sdma_clock >= self.sdma.rate {
                self.sdma_clock = self.sdma_clock.saturating_sub(self.sdma.rate);
                if self.sdma.is_enabled() {
                    self.sdma.start_op();
                }
            }
            if !self.mute {self.samples.lock().unwrap().push(sample)};
        }

        self.display.tick();

        self.cycles += 1;

        if self.cycles == 40704 {
            self.cycles = 0;
            return true;
        }
        return false;
    }

    /// Returns the LCD screen to main
    pub fn get_lcd(&mut self) -> Rc<RefCell<[u8; 3 * 224 * 144]>> {
        Rc::clone(&self.lcd)
    }

    /// A test build used during tests or if the user does not provide a ROM
    pub fn test_build() -> Self {
        let cartridge = Rc::new(RefCell::new(Cartridge::test_build()));
        let io_bus = Rc::new(RefCell::new(IOBus::new(Rc::clone(&cartridge), Vec::new(), None, false, 0)));
        let mem_bus = Rc::new(RefCell::new(MemBus::test_build(Rc::clone(&io_bus), Rc::clone(&cartridge))));
        let cpu = V30MZ::new(Rc::clone(&mem_bus), Rc::clone(&io_bus), false);
        let gdma = GDMA::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let sdma = SDMA::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let sound = Sound::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let lcd = Rc::new(RefCell::new([0; 3 * 224 * 144]));
        let display = Display::new(Rc::clone(&mem_bus), Rc::clone(&io_bus), Rc::clone(&lcd));

        for i in 0..=0x3FFF {
            mem_bus.borrow_mut().write_mem(i, 0x01);
        }
        io_bus.borrow_mut().write_io(0x00, 0xFF);
        io_bus.borrow_mut().write_io(0x1F, 0xF8);

        Self {cpu, gdma, sdma, sound, mem_bus, io_bus, display, cycles: 0, samples: Arc::new(Mutex::new(Vec::new())), sample_acc: 0, sdma_clock: 0, lcd, mute: true}
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod test;