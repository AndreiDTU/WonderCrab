use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{bus::{io_bus::{IOBus, IOBusConnection}, mem_bus::{MemBus, MemBusConnection, Owner}}, cartridge::{Cartridge, Mapper}, cpu::v30mz::V30MZ, display::display_control::Display, dma::DMA, keypad::Keypad};

pub struct SoC {
    // COMPONENTS
    cpu: V30MZ,
    dma: DMA,
    display: Box<Display>,

    // MEMORY
    mem_bus: Rc<RefCell<MemBus>>,

    // I/O
    pub(super) io_bus: Rc<RefCell<IOBus>>,

    cycles: u32,
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
    pub fn new(color: bool, sram: Vec<u8>, rom: Vec<u8>, mapper: Mapper, rewrittable: bool) -> Self {
        let keypad = Rc::new(RefCell::new(Keypad::new()));
        let cartridge = Rc::new(RefCell::new(Cartridge::new(mapper, sram, rom, rewrittable)));
        let io_bus = Rc::new(RefCell::new(IOBus::new(Rc::clone(&cartridge), Rc::clone(&keypad))));
        let mem_bus = Rc::new(RefCell::new(MemBus::new(Rc::clone(&io_bus), Rc::clone(&cartridge))));
        let mut cpu = V30MZ::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let dma = DMA::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let display = Box::new(Display::new(Rc::clone(&mem_bus), Rc::clone(&io_bus)));

        if color {io_bus.borrow_mut().color_setup()}
        cpu.reset();

        Self {cpu, dma, display, mem_bus, io_bus, cycles: 0}
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
        }

        if self.mem_bus.borrow().owner == Owner::CPU {
            return false;
        }

        self.display.tick();

        if self.cycles == 40703 {
            self.cycles = 0;
            return true;
        }
        self.cycles += 1;
        return false;
    }

    pub fn get_lcd(&mut self) -> Rc<RefCell<[(u8, u8, u8); 224 * 144]>> {
        Rc::new(RefCell::new(*self.display.lcd))
    }

    pub fn get_display(&mut self) -> &mut Display {
        &mut self.display
    }
    
    pub fn test_build() -> Self {
        let keypad = Rc::new(RefCell::new(Keypad::new()));
        let cartridge = Rc::new(RefCell::new(Cartridge::test_build()));
        let io_bus = Rc::new(RefCell::new(IOBus::new(Rc::clone(&cartridge), Rc::clone(&keypad))));
        let mem_bus = Rc::new(RefCell::new(MemBus::test_build(Rc::clone(&io_bus), Rc::clone(&cartridge))));
        let cpu = V30MZ::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let dma = DMA::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let display = Display::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));

        for i in 0..=0x3FFF {
            mem_bus.borrow_mut().write_mem(i, 0x01);
        }
        io_bus.borrow_mut().write_io(0x00, 0xFF);
        io_bus.borrow_mut().write_io(0x1F, 0xF8);

        Self {cpu, dma, mem_bus, io_bus, display: Box::new(display), cycles: 0}
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod test;