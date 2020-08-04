use std::io;

/// Manage buffers for a device
pub trait Arena {
    type Buffer: ?Sized;

    /// Allocate buffers
    ///
    /// Returns the number of buffers as reported by the driver.
    ///
    /// # Arguments
    ///
    /// * `count` - Desired number of buffers
    fn allocate(&mut self, count: u32) -> io::Result<u32>;

    /// Release any allocated buffers
    fn release(&mut self) -> io::Result<()>;

    /// Access all buffers
    fn buffers(&self) -> Vec<&Self::Buffer>;

    /// Access a single buffer
    fn get(&self, index: usize) -> Option<&Self::Buffer>;

    /// Access a single buffer without bounds checking
    fn get_unchecked(&self, index: usize) -> &Self::Buffer;
}
