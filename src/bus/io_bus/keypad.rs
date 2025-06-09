use bitflags::bitflags;

bitflags! {
    /// Bitflags representing each button
    #[derive(Copy, Clone, Debug)]
    pub struct Keys: u16 {
        const Y4 = 0x0800;
        const Y3 = 0x0400;
        const Y2 = 0x0200;
        const Y1 = 0x0100;

        const X4 = 0x0080;
        const X3 = 0x0040;
        const X2 = 0x0020;
        const X1 = 0x0010;

        const B = 0x0008;
        const A = 0x0004;
        const Start = 0x0002;
    }
}

/// Contains the state of the console's built-in buttons
pub struct Keypad {
    /// Describes which buttons are currently pressed using a `u16` representing bitflags referring to each button
    state: Keys,
    /// Contains the value emitted to the key scan I/O port
    keys: u8,
}

impl Keypad {
    /// Creates a new object of this type with both fields initialized to 0
    pub fn new() -> Self {
        Self{state: Keys::from_bits_truncate(0), keys: 0}
    }

    /// This polls the keypad asking which keys are currently pressed.
    /// 
    /// The poll parameter is expected to contain bits 4-6 of the key scan I/O port.
    /// These are then used to update the keys field of the struct.
    pub fn poll(&mut self, poll: u8) {
        let action = if poll & 0x04 != 0 {0x0F} else {0x00};
        let x =      if poll & 0x02 != 0 {0x0F} else {0x00};
        let y =      if poll & 0x01 != 0 {0x0F} else {0x00};

        let state = self.state.bits();

        let group = (
            (state & 0x0F) as u8,
            ((state >> 4) & 0x0F) as u8,
            ((state >> 8) & 0x0F) as u8,
        );

        self.keys = (group.0 & action) | (group.1 & x) | (group.2 & y);
    }

    /// Returns the value of the `keys` field
    pub fn read_keys(&self) -> u8 {
        // println!("{:04b}", self.keys);
        self.keys
    }

    #[doc(hidden)]
    pub(super) fn set_key(&mut self, key: Keys, pressed: bool) {
        self.state.set(key, pressed);
    }
}