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

        self.ports[port as usize]
    }

    fn write_io(&mut self, addr: u16, byte: u8) {
        self.ports[addr as usize] = byte
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