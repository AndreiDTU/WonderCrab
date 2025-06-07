use std::{cell::RefCell, rc::Rc};

use crate::bus::{io_bus::{IOBus, IOBusConnection}, mem_bus::{MemBus, MemBusConnection, Owner}};

pub struct DMA {
    mem_bus: Rc<RefCell<MemBus>>,
    io_bus: Rc<RefCell<IOBus>>,

    pub cycles: u8,

    src_addr: u32,
    dest_addr: u16,
    counter: u16,
    dir: bool,
}

impl MemBusConnection for DMA {
    fn read_mem(&mut self, addr: u32) -> u8 {
        self.mem_bus.borrow_mut().read_mem(addr)
    }

    fn write_mem(&mut self, addr: u32, byte: u8) {
        self.mem_bus.borrow_mut().write_mem(addr, byte);
    }
}

impl IOBusConnection for DMA {
    fn read_io(&mut self, addr: u16) -> u8 {
        self.io_bus.borrow_mut().read_io(addr)
    }

    fn write_io(&mut self, addr: u16, byte: u8) {
        self.io_bus.borrow_mut().write_io(addr, byte);
    }
}

impl DMA {
    pub fn new(mem_bus: Rc<RefCell<MemBus>>, io_bus: Rc<RefCell<IOBus>>) -> Self {
        Self {mem_bus, io_bus, cycles: 0, src_addr: 0, dest_addr: 0, counter: 0, dir: false}
    }

    pub fn is_enabled(&mut self) -> bool {
        if !self.io_bus.borrow_mut().color_mode() {return false}
        
        let ctrl = self.read_io(0x48);
        // if ctrl != 0 {println!("DMA ctrl: {:02X}", ctrl)};
        self.dir = ctrl & 0x40 != 0;
        ctrl & 0x80 != 0
    }

    pub fn start_op(&mut self) {
        self.get_counter();
        if self.counter != 0 {
            // println!("counter: {:02X}", self.counter);
            self.get_src_addr();
            match self.src_addr {
                0x10000..=0x1FFFF => return,
                _ => {
                    self.cycles = 7;
                    self.get_dest_addr();
                    self.mem_bus.borrow_mut().owner = Owner::DMA;
                }
            }
        }
    }

    pub fn tick(&mut self) {
        self.cycles -= 1;
        if self.cycles == 0 {
            self.cycles = 2;

            let byte = self.read_mem(self.src_addr);
            self.write_mem(self.dest_addr as u32, byte);

            // println!("DMA: [{:05X}] <- [{:05X}] = {:02X}", self.dest_addr, self.src_addr, byte);

            if self.dir {
                self.src_addr = self.src_addr.wrapping_sub(1);
                self.dest_addr = self.dest_addr.wrapping_sub(1);
            } else {
                self.src_addr = self.src_addr.wrapping_add(1);
                self.dest_addr = self.dest_addr.wrapping_add(1);
            }

            if (0x10000..=0x1FFFF).contains(&self.src_addr) {
                self.cycles = 0;
                self.mem_bus.borrow_mut().owner = Owner::NONE;
            }

            self.counter -= 1;
            if self.counter == 0 {
                self.write_io_16(0x40, self.src_addr as u16);
                self.write_io(0x42, (self.src_addr >> 16) as u8);
                self.write_io_16(0x44, self.dest_addr);
                self.write_io_16(0x46, 0);
                let ctrl = self.read_io(0x48);
                self.write_io(0x48, ctrl & 0x7F);
                self.cycles = 0;
                self.mem_bus.borrow_mut().owner = Owner::NONE;
            }
        }
    }

    fn get_src_addr(&mut self) {
        let (lo, hi) = self.read_io_16(0x40);
        self.src_addr = (((self.read_io(0x42) & 0x0F) as u32) << 16) | u16::from_le_bytes([lo, hi]) as u32;
    }

    fn get_dest_addr(&mut self) {
        let (lo, hi) = self.read_io_16(0x44);
        self.dest_addr = u16::from_le_bytes([lo, hi]);
    }

    fn get_counter(&mut self) {
        let (lo, hi) = self.read_io_16(0x46);
        self.counter = u16::from_le_bytes([lo, hi]);
    }
}