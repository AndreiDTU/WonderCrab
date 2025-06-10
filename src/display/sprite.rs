/// Sprite data
/// 
/// A sprite is a free moving tile of 8x8 pixels, this struct is used to describe sprites
/// 
/// # TODO
/// 
/// Sprite coordinates should wrap around within the visible section of the screen, this is not currently supported
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpriteElement {
    /// Vertical mirroring
    pub vm: bool,
    /// Horizontal mirroring
    pub hm: bool,
    /// Priority, if set will render above screen 2
    pub pr: bool,
    /// Contained, if set and the sprite window is active will only render outside the window, otherwise it will only render inside the window
    pub ct: bool,
    /// Palette index
    pub palette: u8,
    /// Tile index
    pub tile_idx: u16,
    
    /// X coordinate
    pub x: u8,
    /// Y coordinate
    pub y: u8,
}

impl SpriteElement {
    /// Generates new sprite
    pub fn new(vm: bool, hm: bool, pr: bool, ct: bool, palette: u8, tile_idx: u16, x: u8, y: u8) -> Self {
        Self {
            vm, hm, pr, ct,
            palette,
            tile_idx,
            x, y
        }
    }

    /// Generates a dummy sprite for testing or initializing the display
    pub fn dummy() -> Self {
        Self {vm: false, hm: false, pr: false, ct: false, palette: 0, tile_idx: 0, x: 0, y: 0}
    }
}