use crate::assert_eq_hex;

use super::*;

impl SoC {
    pub fn test_build() -> Self {
        let cartridge = Rc::new(RefCell::new(Cartridge::test_build()));
        let io_bus = Rc::new(RefCell::new(IOBus::new(Rc::clone(&cartridge))));
        let mem_bus = Rc::new(RefCell::new(MemBus::test_build(Rc::clone(&io_bus), Rc::clone(&cartridge))));
        let cpu = V30MZ::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let dma = DMA::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));
        let display = Display::new(Rc::clone(&mem_bus), Rc::clone(&io_bus));

        Self {cpu, dma, mem_bus, io_bus, display}
    }

    pub fn set_wram(&mut self, wram: Vec<u8>) {
        for i in 0..wram.len() {
            self.mem_bus.borrow_mut()[i] = wram[i];
        }
    }

    pub fn set_io(&mut self, io: Vec<u8>) {
        for i in 0..io.len() {
            self.io_bus.borrow_mut().write_io(i as u16, io[i]);
        }
    }

    pub fn get_cpu(&mut self) -> &mut V30MZ {
        &mut self.cpu
    }

    pub fn get_wram(&mut self) -> Rc<RefCell<MemBus>> {
        Rc::clone(&self.mem_bus)
    }

    pub fn tick_cpu_no_cycles(&mut self) {
        self.cpu.tick_ignore_cycles();
    }
    
}

#[test]
fn test_io_open_bus() {
    let mut soc = SoC::test_build();
    assert_eq_hex!(soc.read_io(0x100), 0x90);
    assert_eq_hex!(soc.read_io(0x1B9), 0x90);
}