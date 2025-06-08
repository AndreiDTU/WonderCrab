use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}};

use crate::{bus::{io_bus::{IOBus, IOBusConnection}, mem_bus::{MemBus, MemBusConnection, Owner}}, cartridge::{Cartridge, Mapper}, cpu::v30mz::V30MZ, display::display_control::Display, dma::DMA, sound::Sound};

pub struct SoC {
    pub(super) cpu: V30MZ,
    dma: DMA,
    sound: Sound,
    display: Display,

    mem_bus: Rc<RefCell<MemBus>>,
    pub(super) io_bus: Rc<RefCell<IOBus>>,

    cycles: usize,

    pub(super) samples: Arc<Mutex<Vec<(u16, u16)>>>,
    sample_acc: u64,

    lcd: Rc<RefCell<[u8; 3 * 224 * 144]>>,

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
    pub fn new(color: bool, ram_content: Vec<u8>, ieeprom: Vec<u8>, eeprom: Vec<u8>, rom: Vec<u8>, mapper: Mapper, sram: bool, trace: bool, samples: Arc<Mutex<Vec<(u16, u16)>>>, mute: bool, rom_info: u8) -> Self {
        let (cartridge, eeprom) = if sram {
            (Rc::new(RefCell::new(Cartridge::new(mapper, ram_content, rom, sram))), None)
        } else {
            (Rc::new(RefCell::new(Cartridge::new(mapper, Vec::new(), rom, false))), Some(eeprom))
        };
        let io_bus = Rc::new(RefCell::new(IOBus::new(Rc::clone(&cartridge), ieeprom, eeprom, color, rom_info)));
        let mem_bus = Rc::new(RefCell::new(MemBus::new(Rc::clone(&io_bus), Rc::clone(&cartridge))));
        let mut cpu = V30MZ::new(Rc::clone(&mem_bus), Rc::clone(&io_bus), trace);
        let dma = DMA::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let sound = Sound::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let lcd = Rc::new(RefCell::new([0; 3 * 224 * 144]));
        let display = Display::new(Rc::clone(&mem_bus), Rc::clone(&io_bus), Rc::clone(&lcd));

        cpu.reset();

        Self {cpu, dma, sound, display, mem_bus, io_bus, cycles: 0, samples, sample_acc: 0, lcd, mute}
    }

    pub fn tick(&mut self) -> bool {
        if self.dma.cycles == 0 {
            if self.dma.is_enabled() {
                self.dma.start_op();
            }
        }

        if self.dma.cycles > 0 {
            self.dma.tick();
        } else {
            self.cpu.tick();
        };

        if self.mem_bus.borrow().owner == Owner::CPU {
            return false;
        }

        let sample = self.sound.tick();
        if !self.mute {
            self.sample_acc += 1;
            if self.sample_acc >= 128 {
                self.sample_acc -= 128;
                self.samples.lock().unwrap().push(sample);
            }
        }

        self.display.tick();

        self.cycles += 1;

        if self.cycles == 40704 {
            self.cycles = 0;
            return true;
        }
        return false;
    }

    pub fn get_lcd(&mut self) -> Rc<RefCell<[u8; 3 * 224 * 144]>> {
        Rc::clone(&self.lcd)
    }

    pub fn get_display(&mut self) -> &mut Display {
        &mut self.display
    }
    
    pub fn test_build() -> Self {
        let cartridge = Rc::new(RefCell::new(Cartridge::test_build()));
        let io_bus = Rc::new(RefCell::new(IOBus::new(Rc::clone(&cartridge), Vec::new(), None, false, 0)));
        let mem_bus = Rc::new(RefCell::new(MemBus::test_build(Rc::clone(&io_bus), Rc::clone(&cartridge))));
        let cpu = V30MZ::new(Rc::clone(&mem_bus), Rc::clone(&io_bus), false);
        let dma = DMA::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let sound = Sound::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let lcd = Rc::new(RefCell::new([0; 3 * 224 * 144]));
        let display = Display::new(Rc::clone(&mem_bus), Rc::clone(&io_bus), Rc::clone(&lcd));

        for i in 0..=0x3FFF {
            mem_bus.borrow_mut().write_mem(i, 0x01);
        }
        io_bus.borrow_mut().write_io(0x00, 0xFF);
        io_bus.borrow_mut().write_io(0x1F, 0xF8);

        Self {cpu, dma, sound, mem_bus, io_bus, display, cycles: 0, samples: Arc::new(Mutex::new(Vec::new())), sample_acc: 0, lcd, mute: true}
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod test;