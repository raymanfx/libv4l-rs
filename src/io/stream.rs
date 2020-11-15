use std::io;

/// Streaming I/O
pub trait Stream {
    /// Start streaming, takes exclusive ownership of a device
    fn start(&mut self) -> io::Result<()>;

    /// Stop streaming, frees all buffers
    fn stop(&mut self) -> io::Result<()>;
}

pub trait Capture<'a>: Stream {
    type Item;

    /// Queue a new frame on the device
    fn queue(&mut self) -> io::Result<()>;

    /// Read a queued frame back to memory
    fn dequeue(&'a mut self) -> io::Result<Self::Item>;

    /// Fetch a new frame by first queueing and then dequeueing.
    /// First time initialization is performed if necessary.
    fn next(&'a mut self) -> io::Result<Self::Item>;
}

pub trait Output<'a>: Stream {
    type Item;

    /// Queue a new frame on the device
    fn queue(&mut self, item: Self::Item) -> io::Result<()>;

    /// Read a queued frame back to memory
    fn dequeue(&mut self) -> io::Result<()>;

    /// Dump a new frame by first queueing and then dequeueing.
    /// First time initialization is performed if necessary.
    fn next(&mut self, item: Self::Item) -> io::Result<()>;
}
