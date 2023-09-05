use std::io;

use crate::buffer::Metadata;

/// Streaming I/O
pub trait Stream {
    type Item: ?Sized;

    /// Start streaming, takes exclusive ownership of a device
    fn start(&mut self) -> io::Result<()>;

    /// Stop streaming, frees all buffers
    fn stop(&mut self) -> io::Result<()>;
}

pub trait CaptureStream<'a>: Stream {
    /// Insert a buffer into the drivers' incoming queue
    fn queue(&mut self, index: usize) -> io::Result<()>;

    /// Remove a buffer from the drivers' outgoing queue
    fn dequeue(&mut self) -> io::Result<usize>;

    /// Access the buffer at the specified index.
    fn get(&self, index: usize) -> io::Result<(&Self::Item, &Metadata)>;

    /// Fetch a new frame by first queueing and then dequeueing.
    /// First time initialization is performed if necessary.
    fn next(&'a mut self) -> io::Result<(&Self::Item, &Metadata)>;
}

pub trait OutputStream<'a>: Stream {
    /// Insert a buffer into the drivers' incoming queue
    fn queue(&mut self, index: usize) -> io::Result<()>;

    /// Remove a buffer from the drivers' outgoing queue
    fn dequeue(&mut self) -> io::Result<usize>;

    /// Dump a new frame by first queueing and then dequeueing.
    /// First time initialization is performed if necessary.
    fn next(&'a mut self) -> io::Result<(&mut Self::Item, &mut Metadata)>;
}
