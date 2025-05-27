#[derive(Clone, Copy)]
pub struct ScreenElement {
    pub vm: bool,
    pub hm: bool,
    pub palette: u8,
    pub tile_idx: u16,
}

impl ScreenElement {
    pub fn new(vm: bool, hm: bool, palette: u8, tile_idx: u16) -> Self {
        Self {vm, hm, palette, tile_idx}
    }

    pub fn dummy() -> Self {
        Self {vm: false, hm: false, palette: 0, tile_idx: 0}
    }
}