use crate::assert_eq_hex;

use super::*;

impl SoC {
    #[cfg(test)]
    pub fn set_wram(&mut self, wram: Vec<u8>) {
        for i in 0..wram.len() {
            self.mem_bus.borrow_mut()[i] = wram[i];
        }
    }

    #[cfg(test)]
    pub fn set_io(&mut self, io: Vec<u8>) {
        for i in 0..io.len() {
            self.io_bus.borrow_mut().write_io(i as u16, io[i]);
        }
    }

    #[cfg(test)]
    pub fn get_cpu(&mut self) -> &mut V30MZ {
        &mut self.cpu
    }

    #[cfg(test)]
    pub fn get_wram(&mut self) -> Rc<RefCell<MemBus>> {
        Rc::clone(&self.mem_bus)
    }
    
}

#[test]
fn test_io_open_bus() {
    let mut soc = SoC::new();
    assert_eq_hex!(soc.read_io(0x100), 0x90);
    assert_eq_hex!(soc.read_io(0x1B9), 0x90);
}