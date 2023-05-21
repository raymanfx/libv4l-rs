use bitflags::bitflags;
use std::{convert::TryInto, fmt, mem};

use v4l2_sys::v4l2_buffer;

use crate::{memory::Memory, timestamp::Timestamp};

/// Buffer type
///
/// Specific types of devices require buffers of corresponding types.
#[allow(clippy::unreadable_literal)]
#[rustfmt::skip]
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Type {
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

impl Default for Flags {
    fn default() -> Self {
        Flags::from(0)
    }
}

impl From<u32> for Flags {
    fn from(flags: u32) -> Self {
        Self::from_bits_truncate(flags)
    }
}

impl From<Flags> for u32 {
    fn from(flags: Flags) -> Self {
        flags.bits()
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
    /// Number of the buffer
    pub index: u32,
    /// Type of the buffer
    pub type_: u32,
    /// Number of bytes occupied by the data in the buffer
    pub bytesused: u32,
    /// Buffer flags
    pub flags: Flags,
    /// Indicates the field order of the image in the buffer.
    pub field: u32,
    /// Time of capture (usually set by the driver)
    pub timestamp: Timestamp,
    /// Sequence number, counting the frames
    pub sequence: u32,
    /// Memory type, depending on streaming I/O method
    pub memory: Memory,
    /// Single-planar API: size of the buffer (not payload!)
    /// Multi-planar API: number of planes
    pub length: u32,
}

impl From<v4l2_buffer> for Metadata {
    fn from(buf: v4l2_buffer) -> Self {
        Self {
            index: buf.index,
            type_: buf.type_,
            bytesused: buf.bytesused,
            flags: buf.flags.into(),
            field: buf.field,
            timestamp: buf.timestamp.into(),
            sequence: buf.sequence,
            memory: buf.memory.try_into().unwrap(),
            length: buf.length,
        }
    }
}

impl Into<v4l2_buffer> for Metadata {
    fn into(self) -> v4l2_buffer {
        unsafe {
            v4l2_buffer {
                index: self.index,
                type_: self.type_,
                bytesused: self.bytesused,
                flags: self.flags.into(),
                field: self.field,
                timestamp: self.timestamp.into(),
                sequence: self.sequence,
                memory: self.memory as u32,
                length: self.length,
                ..mem::zeroed()
            }
        }
    }
}
