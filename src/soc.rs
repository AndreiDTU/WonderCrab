use std::{cell::RefCell, rc::Rc};

use crate::{bus::{io_bus::{IOBus, IOBusConnection}, mem_bus::{MemBus, MemBusConnection}}, cartridge::{Cartridge, Mapper}, cpu::v30mz::V30MZ, dma::DMA};

pub struct SoC {
    // COMPONENTS
    cpu: V30MZ,
    dma: DMA,

    // MEMORY
    mem_bus: Rc<RefCell<MemBus>>,

    // I/O
    io_bus: Rc<RefCell<IOBus>>,
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
        let cartridge = Rc::new(RefCell::new(Cartridge::new(mapper, sram, rom, rewrittable)));
        let io_bus = Rc::new(RefCell::new(IOBus::new(Rc::clone(&cartridge))));
        let mem_bus = Rc::new(RefCell::new(MemBus::new(Rc::clone(&io_bus), Rc::clone(&cartridge))));
        let mut cpu = V30MZ::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let dma = DMA::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));

        if color {io_bus.borrow_mut().color_setup()}
        cpu.reset();

        Self {cpu, dma, mem_bus, io_bus}
    }

    pub fn run(&mut self, cycles: usize) {
        for _ in 0..cycles {
            self.tick();
        }
    }

    pub fn tick(&mut self) {
        if self.dma.is_enabled() {
            self.dma.start_op();
        }

        if self.dma.cycles > 0 {
            self.dma.tick();
        } else {
            self.cpu.tick();
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod test;