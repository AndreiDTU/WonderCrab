/// An element of the screen's tilemap
#[derive(Clone, Copy, Debug)]
pub struct ScreenElement {
    /// Vertical mirroring
    pub vm: bool,
    /// Horizontal mirroring
    pub hm: bool,
    /// Palette index
    pub palette: u8,
    /// Tile index
    pub tile_idx: u16,
}

impl ScreenElement {
    /// Generates new screen element
    pub fn new(vm: bool, hm: bool, palette: u8, tile_idx: u16) -> Self {
        Self {vm, hm, palette, tile_idx}
    }

    /// Generates a dummy screen element for testing or initializing the display
    pub fn dummy() -> Self {
        Self {vm: false, hm: false, palette: 0, tile_idx: 0}
    }
}