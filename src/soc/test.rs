use crate::assert_eq_hex;

use super::*;

impl SoC {
    #[cfg(test)]
    pub fn set_wram(&mut self, wram: Vec<u8>) {
        for i in 0..wram.len() {
            self.wram.borrow_mut()[i] = wram[i];
        }
    }

    #[cfg(test)]
    pub fn set_io(&mut self, io: Vec<u8>) {
        for i in 0..io.len() {
            self.io.borrow_mut()[i] = io[i];
        }
    }

    #[cfg(test)]
    pub fn get_cpu(&mut self) -> &mut V30MZ {
        &mut self.cpu
    }

    #[cfg(test)]
    pub fn get_wram(&mut self) -> Rc<RefCell<[u8; 0x10000]>> {
        Rc::clone(&self.wram)
    }
    
}

#[test]
fn test_io_open_bus() {
    let mut soc = SoC::new();
    assert_eq_hex!(soc.read_io(0x100), 0x90);
    assert_eq_hex!(soc.read_io(0x1B9), 0x90);
}