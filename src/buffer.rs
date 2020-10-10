use bitflags::bitflags;
use std::{fmt, io, marker, ops};

use crate::timestamp::Timestamp;

/// Buffer type
///
/// Specific types of devices require buffers of corresponding types.
#[allow(clippy::unreadable_literal)]
#[rustfmt::skip]
#[repr(u32)]
pub enum BufferType {
    VideoCapture        = 1,
    VideoOutput         = 2,
    VideoOverlay        = 3,
    VbiCaputre          = 4,
    VbiOutput           = 5,
    SlicedVbiCapture    = 6,
    SlicedVbiOutput     = 7,
    VideoOutputOverlay  = 8,
    VideoCaptureMplane  = 9,
    VideoOutputMplane   = 10,
    SdrCapture          = 11,
    SdrOutput           = 12,
    MetaCapture         = 13,
    MetaOutput          = 14,

    /// Deprecated, do not use
    Private             = 0x80,
}

bitflags! {
    #[allow(clippy::unreadable_literal)]
    pub struct Flags: u32 {
        /// Buffer is mapped
        const MAPPED                = 0x00000001;
        /// Buffer is queued for processing
        const QUEUED                = 0x00000002;
        /// Buffer is ready
        const DONE                  = 0x00000004;
        /// Image is a keyframe (I-frame)
        const KEYFRAME              = 0x00000008;
        /// Image is a P-frame
        const PFRAME                = 0x00000010;
        /// Image is a B-frame
        const BFRAME                = 0x00000020;
        /// Buffer is ready, but the data contained within is corrupted
        const ERROR                 = 0x00000040;
        /// Buffer is added to an unqueued request
        const IN_REQUEST            = 0x00000080;
        /// Timecode field is valid
        const TIMECODE              = 0x00000100;
        /// Don't return the capture buffer until OUTPUT timestamp changes
        const M2M_HOLD_CAPTURE_BUF  = 0x00000200;
        /// Buffer is prepared for queuing
        const PREPARED              = 0x00000400;
        /// Cache handling flags
        const NO_CACHE_INVALIDATE   = 0x00000800;
        const NO_CACHE_CLEAN        = 0x00001000;
        /// Timestamp type
        const TIMESTAMP_MASK        = 0x0000e000;
        const TIMESTAMP_UNKNOWN     = 0x00000000;
        const TIMESTAMP_MONOTONIC   = 0x00002000;
        const TIMESTAMP_COPY        = 0x00004000;
        /// Timestamp sources
        const TSTAMP_SRC_MASK       = 0x00070000;
        const TSTAMP_SRC_EOF        = 0x00000000;
        const TSTAMP_SRC_SOE        = 0x00010000;
        /// mem2mem encoder/decoder
        const LAST                  = 0x00100000;
        /// request_fd is valid
        const REQUEST_FD            = 0x00800000;
    }
}

impl From<u32> for Flags {
    fn from(flags: u32) -> Flags {
        Flags::from_bits_truncate(flags)
    }
}

impl Into<u32> for Flags {
    fn into(self) -> u32 {
        self.bits()
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// Buffer metadata, mostly used not to convolute the main buffer structs
#[derive(Copy, Clone)]
pub struct Metadata {
    /// Number of bytes occupied by the data in the buffer
    pub bytesused: u32,
    /// Buffer flags
    pub flags: Flags,
    /// Time of capture (usually set by the driver)
    pub timestamp: Timestamp,
    /// Sequence number, counting the frames
    pub sequence: u32,
}

/// Represents a buffer view
pub struct Buffer<'a> {
    view: &'a [u8],
    metadata: Metadata,
}

impl<'a> Buffer<'a> {
    /// Returns a memory region view
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
    /// use v4l::{buffer, timestamp};
    ///
    /// let data: Vec<u8> = Vec::new();
    /// let ts = timestamp::Timestamp::new(0 /* sec */, 0 /* usec */);
    /// let flags = buffer::Flags::from(0);
    /// let meta = buffer::Metadata {
    ///     bytesused: 0,
    ///     flags,
    ///     timestamp: ts,
    ///     sequence: 0,
    /// };
    /// let buf = buffer::Buffer::new(&data, meta);
    /// ```
    pub fn new(view: &'a [u8], meta: Metadata) -> Self {
        Buffer {
            view,
            metadata: meta,
        }
    }

    /// Slice of read-only data
    pub fn data(&self) -> &[u8] {
        self.view
    }

    /// Metadata such as allocation flags, timestamp and more
    pub fn meta(&self) -> &Metadata {
        &self.metadata
    }
}

impl<'a> ops::Deref for Buffer<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.view
    }
}

/// Streaming I/O
pub trait Stream<'a> {
    type Item;

    /// Start streaming, takes exclusive ownership of a device
    fn start(&mut self) -> io::Result<()>;

    /// Stop streaming, frees all buffers
    fn stop(&mut self) -> io::Result<()>;

    /// Queue a new frame on the device
    fn queue(&mut self) -> io::Result<()>;

    /// Read a queued frame back to memory
    fn dequeue(&'a mut self) -> io::Result<StreamItem<'a, Self::Item>>;

    /// Fetch a new frame by first queueing and then dequeueing.
    /// First time initialization is performed if necessary.
    fn next(&'a mut self) -> io::Result<StreamItem<'a, Self::Item>>;
}

/// Stream item wrapper
///
/// The sole purpose of this wrapper struct is to attach a lifetime to values of type T.
/// This is especially useful for volatile types such as views which provide access to some kind of
/// underlying data.
pub struct StreamItem<'a, T> {
    /// The wrapped item
    item: T,
    // Used to augment the item with a lifetime to benefit from the borrow checker
    _lifetime: marker::PhantomData<&'a mut ()>,
}

impl<'a, T> StreamItem<'a, T> {
    /// Returns a wrapped stream item by moving it into the wrapper
    ///
    /// An explicit lifetime is attached automatically by inserting PhantomData.
    ///
    /// # Arguments
    ///
    /// * `item` - Item to be wrapped
    ///
    /// # Example
    ///
    /// ```
    /// use std::ops::Deref;
    /// use v4l::buffer::StreamItem;
    ///
    /// let item: u32 = 123;
    /// let wrapper = StreamItem::new(item);
    /// assert_eq!(*wrapper.deref(), item);
    /// ```
    pub fn new(item: T) -> Self {
        StreamItem {
            item,
            _lifetime: marker::PhantomData,
        }
    }
}

impl<'a, T> ops::Deref for StreamItem<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}
