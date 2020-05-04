use crate::{buffer, Timestamp};

/// Buffer allocated in userspace (by the application)
///
/// Devices supporting user pointer mode will directly transfer image memory to the buffer
/// "for free" by using direct memory access (DMA).
pub struct UserBuffer<'a> {
    flags: buffer::Flags,
    timestamp: Timestamp,
    sequence: u32,

    view: &'a [u8],
}

impl<'a> UserBuffer<'a> {
    /// Returns a user buffer representation
    ///
    /// Buffers created this way provide read-only access to the backing data, just like mapped
    /// ones. This is necessary because the MappedBufferManager allocated a number of buffers once
    /// and then keeps them allocated for the lifetime of the stream. Otherwise, we would have to
    /// allocate a new buffer for every frame, which is costly.
    ///
    /// If you need to alter the data in this buffer, copy it out of the slice into your own
    /// container type and process it.
    ///
    /// # Arguments
    ///
    /// * `view` - Slice of raw memory
    /// * `seq` - Sequence number as counted by the driver
    /// * `ts` - Timestamp as reported by the driver
    /// * `flags` - Flags as set by the driver
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::{buffer, Timestamp, UserBuffer};
    ///
    /// let data: Vec<u8> = Vec::new();
    /// let ts = Timestamp::new(0 /* sec */, 0 /* usec */);
    /// let flags = buffer::Flags::from(0);
    /// let buf = UserBuffer::new(&data, 0, ts, flags);
    /// ```
    pub fn new(view: &'a [u8], seq: u32, ts: Timestamp, flags: buffer::Flags) -> Self {
        UserBuffer {
            flags,
            timestamp: ts,
            sequence: seq,
            view,
        }
    }
}

impl<'a> buffer::Buffer for UserBuffer<'a> {
    fn data(&self) -> &[u8] {
        &self.view
    }

    fn len(&self) -> usize {
        self.view.len()
    }

    fn is_empty(&self) -> bool {
        self.view.is_empty()
    }

    fn seq(&self) -> u32 {
        self.sequence
    }

    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    fn flags(&self) -> buffer::Flags {
        self.flags
    }
}
