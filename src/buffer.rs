use bitflags::bitflags;
use std::{fmt, io};

use crate::Timestamp;

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
    /// Sequence number, counting the frames
    pub seq: u32,
    /// Time of capture (usually set by the driver)
    pub timestamp: Timestamp,
    /// Buffer flags
    pub flags: Flags,
}

impl Metadata {
    /// Returns a buffer metadata description
    ///
    /// # Arguments
    ///
    /// * `seq` - Sequence number as counted by the driver
    /// * `ts` - Timestamp as reported by the driver
    /// * `flags` - Flags as set by the driver
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::{buffer, Timestamp};
    ///
    /// let ts = Timestamp::new(0 /* sec */, 0 /* usec */);
    /// let flags = buffer::Flags::from(0);
    /// let meta = buffer::Metadata::new(0, ts, flags);
    /// ```
    pub fn new(seq: u32, ts: Timestamp, flags: Flags) -> Self {
        Metadata {
            seq,
            timestamp: ts,
            flags,
        }
    }
}

/// Represents a host (allocated) or device (mapped) buffer
pub trait Buffer {
    /// Slice of read-only data
    fn data(&self) -> &[u8];

    /// Size of the backing memory region
    fn len(&self) -> usize;

    /// Whether the backing buffer is empty
    fn is_empty(&self) -> bool;

    /// Metadata such as allocation flags, timestamp and more
    fn meta(&self) -> &Metadata;
}

/// Manage buffers for a device
pub trait Arena {
    /// Type of the buffers (DMA, mmap, userptr)
    type Buffer;

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

    /// Queue a new buffer on the device
    ///
    /// Queueing usually causes the camera hardware to take a picture.
    /// Dequeuing then transfers it to application visible memory.
    fn queue(&mut self) -> io::Result<()>;

    /// Dequeue a buffer and return it to the application
    fn dequeue(&mut self) -> io::Result<Self::Buffer>;
}

/// Streaming I/O
pub trait Stream: Iterator {
    type Buffer;

    /// Whether the stream is currently active
    fn active(&self) -> bool;

    /// Start streaming, takes exclusive ownership of a device
    fn start(&mut self) -> io::Result<()>;

    /// Stop streaming, frees all buffers
    fn stop(&mut self) -> io::Result<()>;

    /// Queue a new frame on the device
    fn queue(&mut self) -> io::Result<()>;

    /// Read a queued frame back to memory
    fn dequeue(&mut self) -> io::Result<Self::Buffer>;
}

/// Iterate through a (possibly endless) stream of buffers
///
/// This works more like a generator rather than a classic iterator because frames are captured as
/// they are requested (on the fly). Once an error condition occurs, None is returned so the caller
/// can know about a broken stream.
pub struct StreamIterator<'a, S: Stream> {
    /// Mutable stream reference representing exclusive ownership
    stream: &'a mut S,
}

impl<'a, S: Stream> StreamIterator<'a, S> {
    pub fn new(stream: &'a mut S) -> Self {
        StreamIterator { stream }
    }
}

impl<'a, S: Stream> Iterator for StreamIterator<'a, S> {
    type Item = S::Buffer;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.stream.active() && self.stream.start().is_err() {
            return None;
        }

        let buf = self.stream.dequeue();
        if buf.is_err() {
            return None;
        }

        if self.stream.queue().is_err() {
            return None;
        }

        Some(buf.unwrap())
    }
}
