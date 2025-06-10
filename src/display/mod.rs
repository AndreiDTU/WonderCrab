/// Core display module
/// 
/// This module is public so that main can send the contents of the frame to SDL for display
pub mod display_control;
/// Contains information related to screen elements
mod screen;
/// Contains information related to sprites
mod sprite;

/// Format encoding the color index of each pixel within the tile's palette
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PaletteFormat {
    /// 2 bits per pixel, each pair of bytes describes the low and high bit respectively of a row of 8 pixels, 16 bytes per tile
    PLANAR_2BPP,
    /// 4 bits per pixel, each quadruplet of bytes describes the bits from low to high of a row of 8 pixels, 32 bytes per tile
    PLANAR_4BPP,
    /// 4 bits per pixel, each nibble of each byte describes a given pixel, 32 bytes per tile
    PACKED_4BPP,
}