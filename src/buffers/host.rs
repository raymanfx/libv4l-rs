use crate::buffer;

/// Represents a host buffer backed by RAM
pub struct HostBuffer {
    backing: Vec<u8>,
    metadata: buffer::Metadata,
}

impl HostBuffer {
    /// Returns a buffer backed by host memory (RAM)
    ///
    /// The `view` slice is copied by cloning its bytes into a backing storage
    /// container. This means a `HostBuffer` can always be passed around
    /// freely with no associated lifetimes.
    ///
    /// # Arguments
    ///
    /// * `view` - Slice of raw memory
    /// * `meta` - Metadata, usually filled in by the driver
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::{buffer, HostBuffer, Timestamp};
    ///
    /// let data: Vec<u8> = Vec::new();
    /// let ts = Timestamp::new(0 /* sec */, 0 /* usec */);
    /// let flags = buffer::Flags::from(0);
    /// let meta = buffer::Metadata::new(0, ts, flags);
    /// let buf = HostBuffer::new(&data, meta);
    /// ```
    pub fn new(view: &[u8], meta: buffer::Metadata) -> Self {
        HostBuffer {
            backing: view.into(),
            metadata: meta,
        }
    }

    pub fn from<B: buffer::Buffer>(buf: &B) -> Self {
        HostBuffer::new(buf.data(), *buf.meta())
    }
}

impl buffer::Buffer for HostBuffer {
    fn data(&self) -> &[u8] {
        &self.backing
    }

    fn len(&self) -> usize {
        self.backing.len()
    }

    fn is_empty(&self) -> bool {
        self.backing.is_empty()
    }

    fn meta(&self) -> &buffer::Metadata {
        &self.metadata
    }
}
