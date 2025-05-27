pub struct SpriteElement {
    pub vm: bool,
    pub hm: bool,
    pub pr: bool,
    pub ct: bool,
    pub palette: u8,
    pub tile: [[u8; 8]; 8],
    pub x: u8, pub y: u8,
}

impl SpriteElement {
    pub fn new(vm: bool, hm: bool, pr: bool, ct: bool, palette: u8, tile: [[u8; 8]; 8], x: u8, y: u8) -> Self {
        Self {
            vm, hm, pr, ct,
            palette,
            tile,
            x, y
        }
    }
}