use std::{cell::RefCell, rc::Rc};

use crate::{bus::{io_bus::{IOBus, IOBusConnection}, mem_bus::{MemBus, MemBusConnection}}, cpu::v30mz::V30MZ};

pub struct SoC {
    // COMPONENTS
    cpu: V30MZ,

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
    pub fn new() -> Self {
        let io_bus = Rc::new(RefCell::new(IOBus::new()));
        let mem_bus = Rc::new(RefCell::new(MemBus::new(Rc::clone(&io_bus))));
        let cpu = V30MZ::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));

        Self {cpu, mem_bus, io_bus}
    }

    pub fn tick(&mut self) {
        self.cpu.tick();
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod test;