use crate::{buffer, Timestamp};

/// Memory mapped buffer
///
/// The buffer is backed by camera device or kernel memory.
/// Read only access (e.g. by directly uploading it to the GPU) is permitted for the lifetime of
/// the buffer instance.
/// Acquiring ownership of the data in userspace is not possible, so it has to be copied.
pub struct MappedBuffer<'a> {
    flags: buffer::Flags,
    timestamp: Timestamp,
    sequence: u32,

    view: &'a [u8],
}

impl<'a> MappedBuffer<'a> {
    /// Returns a mapped memory region representation
    ///
    /// Buffers created this way provide read-only access to the backing data, enforcing callers
    /// to copy the data before mutating it.
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
    /// use v4l::{buffer, MappedBuffer, Timestamp};
    ///
    /// let data: Vec<u8> = Vec::new();
    /// let ts = Timestamp::new(0 /* sec */, 0 /* usec */);
    /// let flags = buffer::Flags::from(0);
    /// let buf = MappedBuffer::new(&data, 0, ts, flags);
    /// ```
    pub fn new(view: &'a [u8], seq: u32, ts: Timestamp, flags: buffer::Flags) -> Self {
        MappedBuffer {
            flags,
            timestamp: ts,
            sequence: seq,
            view,
        }
    }
}

impl<'a> buffer::Buffer for MappedBuffer<'a> {
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
