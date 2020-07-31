use crate::buffer;

/// Buffer allocated in userspace (by the application)
///
/// Devices supporting user pointer mode will directly transfer image memory to the buffer
/// "for free" by using direct memory access (DMA).
pub struct Buffer<'a> {
    view: &'a [u8],
    metadata: buffer::Metadata,
}

impl<'a> Buffer<'a> {
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
    /// * `meta` - Metadata, usually filled in by the driver
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::{buffer, Timestamp, io::userptr::Buffer};
    ///
    /// let data: Vec<u8> = Vec::new();
    /// let ts = Timestamp::new(0 /* sec */, 0 /* usec */);
    /// let flags = buffer::Flags::from(0);
    /// let meta = buffer::Metadata::new(0, ts, flags);
    /// let buf = Buffer::new(&data, meta);
    /// ```
    pub fn new(view: &'a [u8], meta: buffer::Metadata) -> Self {
        Buffer {
            view,
            metadata: meta,
        }
    }
}

impl<'a> buffer::Buffer for Buffer<'a> {
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
