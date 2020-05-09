use crate::buffer;

/// Memory mapped buffer
///
/// The buffer is backed by camera device or kernel memory.
/// Read only access (e.g. by directly uploading it to the GPU) is permitted for the lifetime of
/// the buffer instance.
/// Acquiring ownership of the data in userspace is not possible, so it has to be copied.
pub struct MappedBuffer<'a> {
    view: &'a [u8],
    metadata: buffer::Metadata,
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
    /// * `meta` - Metadata, usually filled in by the driver
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::{buffer, MappedBuffer, Timestamp};
    ///
    /// let data: Vec<u8> = Vec::new();
    /// let ts = Timestamp::new(0 /* sec */, 0 /* usec */);
    /// let flags = buffer::Flags::from(0);
    /// let meta = buffer::Metadata::new(0, ts, flags);
    /// let buf = MappedBuffer::new(&data, meta);
    /// ```
    pub fn new(view: &'a [u8], meta: buffer::Metadata) -> Self {
        MappedBuffer {
            view,
            metadata: meta,
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

    fn meta(&self) -> &buffer::Metadata {
        &self.metadata
    }
}
