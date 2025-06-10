use std::{cell::RefCell, rc::Rc};

use crate::{bus::{io_bus::{IOBus, IOBusConnection}, mem_bus::{MemBus, MemBusConnection}}, dma::DMA};

/// Sound DMA
/// 
/// This component is used for transferring 8-bit audio samples into channel 2, used mainly for voice clips
pub struct SDMA {
    /// A reference to the shared memory bus
    mem_bus: Rc<RefCell<MemBus>>,
    /// A reference to the shared I/O bus
    io_bus: Rc<RefCell<IOBus>>,

    /// Cycles until the current transfer completes
    pub cycles: u8,

    /// Source address, the address from which the DMA should read
    src_addr: u32,
    /// Amount of bytes to be transferred
    counter: u32,

    /// Shadow of the source address
    src_shadow: u32,
    /// Shadow of the counter
    counter_shadow: u32,

    /// Direction flag
    /// 
    /// If set the addresses will be decremented after each transfer, otherwise they will be incremented.
    dir: bool,
    /// Repeat flag
    /// 
    /// If set, once the counter has ticked down to 0, the counter and source address will be reset based on their shadows
    rep: bool,
    /// Hold flag
    /// 
    /// If set, the counter and source will not be changed on each tick and the DMA will output 0
    hold: bool,
    /// The rate at which samples should change
    /// 
    /// Samples change at 24kHz / rate or once every 128 * rate ticks
    pub rate: u8,

    /// Running flag
    /// 
    /// Is set at the start of each operation. Is cleared at the end of each operation.
    /// 
    /// If not set, then the source address and counter getters will also write to the shadow.
    running: bool,
}

impl MemBusConnection for SDMA {
    fn read_mem(&mut self, addr: u32) -> u8 {
        self.mem_bus.borrow_mut().read_mem(addr)
    }

    fn write_mem(&mut self, addr: u32, byte: u8) {
        self.mem_bus.borrow_mut().write_mem(addr, byte);
    }
}

impl IOBusConnection for SDMA {
    fn read_io(&mut self, addr: u16) -> u8 {
        self.io_bus.borrow_mut().read_io(addr)
    }

    fn write_io(&mut self, addr: u16, byte: u8) {
        self.io_bus.borrow_mut().write_io(addr, byte);
    }
}

impl DMA for SDMA {
    fn is_enabled(&mut self) -> bool {
        if !self.io_bus.borrow_mut().color_mode() {return false}

        let ctrl = self.read_io(0x52);
        self.dir = ctrl & 0x40 != 0;
        self.rep = ctrl & 0x08 != 0;
        self.hold = ctrl & 0x04 != 0;
        self.rate = match ctrl & 3 {
            0 => 6,
            1 => 4,
            2 => 2,
            3 => 1,
            _ => unreachable!()
        };
        ctrl & 0x80 != 0
    }

    fn start_op(&mut self) {
        self.get_counter();
        if self.counter != 0 {
            self.get_src_addr();
            self.cycles = 7;
            self.running = true;
        }
    }

    fn tick(&mut self) {
        self.cycles -= 1;
        if self.cycles == 0 {
            if self.hold {
                self.write_io(0x89, 0x00);
            } else {
                let byte = self.read_mem(self.src_addr);
                self.write_io(0x89, byte);

                self.src_addr = if self.dir {
                    self.src_addr.wrapping_sub(1)
                } else {
                    self.src_addr.wrapping_add(1)
                };

                self.counter -= 1;
                if self.counter == 0 {
                    if self.rep {
                        self.counter = self.counter_shadow;
                        self.src_addr = self.src_shadow;
                    } else {
                        let ctrl = self.read_io(0x52);
                        self.write_io(0x52, ctrl & 0x7F);
                        self.running = false;
                    }
                }

                self.write_counter();
                self.write_src_addr();
            }
        }
    }
}

impl SDMA {
    /// Generates a new SDMA
    pub fn new(mem_bus: Rc<RefCell<MemBus>>, io_bus: Rc<RefCell<IOBus>>) -> Self {
        Self {
            mem_bus, io_bus,
            cycles: 0,
            
            src_addr: 0, counter: 0,
            src_shadow: 0, counter_shadow: 0,
            
            dir: false, rep: false, hold: false,
            rate: 1,
            
            running: false
        }
    }

    /// Reads the counter from the appropriate I/O ports
    fn get_counter(&mut self) {
        let (lo, hi) = self.read_io_16(0x4E);
        let offset = u16::from_le_bytes([lo, hi]) as u32;
        let segment = (self.read_io(0x50) & 0x0F) as u32;
        self.counter = (segment << 16) | offset;
        if !self.running {self.counter_shadow = self.counter};
    }

    /// Writes the counter to the appropriate I/O ports
    fn write_counter(&mut self) {
        let offset = self.counter as u16;
        let segment = ((self.counter >> 16) & 0x0F) as u8;
        self.write_io_16(0x4E, offset);
        self.write_io(0x50, segment);
    }

    /// Reads the source address from the appropriate I/O ports
    fn get_src_addr(&mut self) {
        let (lo, hi) = self.read_io_16(0x4A);
        let offset = u16::from_le_bytes([lo, hi]) as u32;
        let segment = (self.read_io(0x4C) & 0x0F) as u32;
        self.src_addr = (segment << 16) | offset;
        if !self.running {self.src_shadow = self.src_addr};
    }

    /// Writes the source address to the appropriate I/O port
    fn write_src_addr(&mut self) {
        let offset = self.counter as u16;
        let segment = ((self.counter >> 16) & 0x0F) as u8;
        self.write_io_16(0x4A, offset);
        self.write_io(0x4C, segment);
    }
}