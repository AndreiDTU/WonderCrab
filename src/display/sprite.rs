#[derive(Clone, Copy)]
pub struct SpriteElement {
    pub vm: bool,
    pub hm: bool,
    pub pr: bool,
    pub ct: bool,
    pub palette: u8,
    pub tile_idx: u16,
    pub x: u8, pub y: u8,
}

impl SpriteElement {
    pub fn new(vm: bool, hm: bool, pr: bool, ct: bool, palette: u8, tile_idx: u16, x: u8, y: u8) -> Self {
        Self {
            vm, hm, pr, ct,
            palette,
            tile_idx,
            x, y
        }
    }

    pub fn dummy() -> Self {
        Self {vm: false, hm: false, pr: false, ct: false, palette: 0, tile_idx: 0, x: 0, y: 0}
    }
}