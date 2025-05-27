pub struct ScreenElement {
    pub vm: bool,
    pub hm: bool,
    pub palette: u8,
    pub tile: [[u8; 8]; 8]
}

impl ScreenElement {
    pub fn new(vm: bool, hm: bool, palette: u8, tile: [[u8; 8]; 8]) -> Self {
        Self {vm, hm, palette, tile}
    }
}