pub struct IOBus {
    ports: [u8; 0x100],
}

pub trait IOBusConnection {
    fn read_io(&mut self, addr: u16) -> u8;
    fn write_io(&mut self, addr: u16, byte: u8);

    fn read_io_16(&mut self, addr: u16) -> (u8, u8) {
        (self.read_io(addr), self.read_io(addr.wrapping_add(1)))
    }
}

impl IOBusConnection for IOBus {
    fn read_io(&mut self, addr: u16) -> u8 {
        if addr & 0x0100 != 0 {
            return 0x90
        }

        let port = addr as u8;
        if addr > 0xFF && port > 0xB8 {
            return 0x90
        }

        match port {
            // INT_CAUSE_CLEAR is write-only
            0xB6 => {0}

            // INT_NMI_CTRL clears most of its bits when read
            0xB7 => {
                self.ports[0xB7] &= 0x10;
                self.ports[0xB7]
            }

            // Default no side-effects
            _ => self.ports[port as usize]
        }
    }

    fn write_io(&mut self, addr: u16, byte: u8) {
        if addr & 0x0100 != 0 {
            return;
        }

        let port = addr as u8;
        if addr > 0xFF && port > 0xB8 {
            return;
        }

        match port {
            // INT_CAUSE is read-only
            0xB4 => {}

            // INT_CAUSE_CLEAR clears bits of INT_CAUSE when written to
            0xB6 => {
                self.ports[0xB6] = byte;
                self.ports[0xB4] &= !self.ports[0xB6]
            }

            // Default no side-effects
            _ => self.ports[port as usize] = byte
        }
    }
}

impl IOBus {
    pub fn new() -> Self {
        Self {ports: [0; 0x100]}
    }

    pub fn color_mode(&mut self) -> bool {
        self.read_io(0x60) >> 7 != 0
    }
}