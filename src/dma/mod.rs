/// General DMA
pub mod gdma;
/// Sound DMA
pub mod sdma;

/// A trait to be implemented by the DMAs
pub trait DMA {
    /// Reads the DMA's control port, sets the appropriate fields and returns whether or not the port is enabled
    fn is_enabled(&mut self) -> bool;
    /// Finds the data needed to start an operation that is not contained in the control port and starts an operation if one is possible
    fn start_op(&mut self);
    /// Ticks the DMA by one cycle.
    /// 
    /// DMAs do not receive their own master clock quadrant and instead hijack the CPU's quadrant
    fn tick(&mut self);
}