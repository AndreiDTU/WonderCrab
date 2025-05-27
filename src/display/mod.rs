pub mod display_control;
mod screen;
mod sprite;

#[derive(Clone, Copy)]
pub enum PaletteFormat {
    PLANAR_2BPP,
    PLANAR_4BPP,
    PACKED_4BPP,
}