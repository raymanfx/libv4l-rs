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

/// Buffer flags
#[allow(clippy::unreadable_literal)]
#[rustfmt::skip]
#[repr(u32)]
pub enum BufferFlag {
    /// Buffer is mapped
    Mapped              = 0x00000001,
    /// Buffer is queued for processing
    Queued              = 0x00000002,
    /// Buffer is ready
    Done                = 0x00000004,
    /// Image is a keyframe (I-frame)
    Keyframe            = 0x00000008,
    /// Image is a P-frame
    Pframe              = 0x00000010,
    /// Image is a B-frame
    Bframe              = 0x00000020,
    /// Buffer is ready, but the data contained within is corrupted
    Error               = 0x00000040,
    /// Buffer is added to an unqueued request
    InRequest           = 0x00000080,
    /// Timecode field is valid
    Timecode            = 0x00000100,
    /// Don't return the capture buffer until OUTPUT timestamp changes
    M2MHoldCaptureBuf   = 0x00000200,
    /// Buffer is prepared for queuing
    Prepared            = 0x00000400,
    /// Cache handling flags
    NoCacheInvalidate   = 0x00000800,
    NoCacheClean        = 0x00001000,
    /// Timestamp type
    TimestampMask       = 0x0000e000,
    //TimestampUnknown    = 0x00000000,
    TimestampMonotonic  = 0x00002000,
    TimestampCopy       = 0x00004000,
    TstampMask          = 0x00070000,
    //TstampSrcEof        = 0x00000000,
    TstampSrcSoe        = 0x00010000,
    /// mem2mem encoder/decoder
    Last                = 0x00100000,
    /// request_fd is valid
    RequestFd           = 0x00800000,
}

#[derive(Debug, Default, Copy, Clone)]
/// Buffer flags
pub struct BufferFlags {
    /// Buffer flags such as V4L2_BUF_FLAG_MAPPED
    pub flags: u32,
}

impl From<u32> for BufferFlags {
    fn from(flags: u32) -> Self {
        BufferFlags { flags }
    }
}

impl fmt::Display for BufferFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut prefix = "";
        let mut flags = self.flags;

        let mut print_flag = |flag: BufferFlag, info: &str| -> fmt::Result {
            let flag = flag as u32;
            if flags & flag != 0 {
                write!(f, "{}{}", prefix, info)?;
                prefix = ", ";

                // remove from input flags so we can know about flags we do not recognize
                flags &= !flag;
            }
            Ok(())
        };

        print_flag(BufferFlag::Mapped, "mapped")?;
        print_flag(BufferFlag::Queued, "queued")?;
        print_flag(BufferFlag::Done, "ready")?;
        print_flag(BufferFlag::Keyframe, "I-frame")?;
        print_flag(BufferFlag::Pframe, "P-frame")?;
        print_flag(BufferFlag::Bframe, "B-frame")?;
        print_flag(BufferFlag::Error, "corruped")?;
        print_flag(BufferFlag::InRequest, "in request")?;
        print_flag(BufferFlag::Timecode, "timecode")?;
        print_flag(BufferFlag::M2MHoldCaptureBuf, "hold")?;
        print_flag(BufferFlag::Prepared, "prepared for queuing")?;
        print_flag(BufferFlag::NoCacheInvalidate, "no cache invalidate")?;
        print_flag(BufferFlag::NoCacheClean, "no cache clean")?;
        print_flag(BufferFlag::TimestampMask, "timestamp")?;
        print_flag(BufferFlag::TimestampMonotonic, "monotonic timestamp")?;
        print_flag(BufferFlag::TimestampCopy, "copied timestamp")?;
        print_flag(BufferFlag::TstampMask, "TstampMask")?;
        print_flag(BufferFlag::TstampSrcSoe, "TstampSrcSoe")?;
        print_flag(BufferFlag::Last, "last")?;
        print_flag(BufferFlag::RequestFd, "request fd valid")?;

        if flags != 0 {
            write!(f, "{}{}", prefix, flags)?;
        }
        Ok(())
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

    /// Sequence number of the buffer, counting the frames
    fn seq(&self) -> u32;

    /// Time of capture (usually set by the driver)
    fn timestamp(&self) -> Timestamp;

    /// Buffer flags
    fn flags(&self) -> BufferFlags;
}

/// Manage buffers for a device
pub trait BufferManager {
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
pub trait BufferStream: Iterator {
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
pub struct StreamIterator<'a, S: BufferStream> {
    /// Mutable stream reference representing exclusive ownership
    stream: &'a mut S,
}

impl<'a, S: BufferStream> StreamIterator<'a, S> {
    pub fn new(stream: &'a mut S) -> Self {
        StreamIterator { stream }
    }
}

impl<'a, S: BufferStream> Iterator for StreamIterator<'a, S> {
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
