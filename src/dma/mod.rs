pub mod gdma;
pub mod sdma;
pub trait DMA {
    fn is_enabled(&mut self) -> bool;
    fn start_op(&mut self);
    fn tick(&mut self);
}