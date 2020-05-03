//! This crate provides safe bindings for the Video4Linux (v4l) stack.
//!
//! The stack consists of three libraries written in C:
//! * libv4l1       (v4l1 API, deprecated)
//! * libv4l2       (v4l2 API, the primary target of this crate)
//! * libv4lconvert (emulates common formats such as RGB3 in userspace)
//!
//! Additional documentation can currently also be found in the
//! [README.md file which is most easily viewed on github](https://github.com/raymanfx/libv4l-rs/blob/master/README.md).
//!
//! [Jump forward to crate content](#reexports)
//!
//! # Overview
//!
//! Video devices on Linux can be accessed by path or by index (which then corresponds to a path),
//! e.g. "/dev/video0" for the device which first became known to the system.
//!
//! There are three methods of dealing with (capture) device memory:
//! * MMAP (memory region in device memory or kernel space, mapped into userspace)
//! * User pointer (memory region allocated in host memory, written into by the kernel)
//! * DMA (direct memory access for memory transfer without involving the CPU)
//!
//! The following schematic shows the mmap and userptr mechanisms:
//!
//! **mmap**
//!
//! 1. device --[MAP]--> kernel --[MAP]--> user
//! 2. device --[DMA]--> kernel --[MAP]--> user
//!
//! **userptr**
//!
//! 3. device --[DMA]-->                   user
//!
//!
//! It is important to note that user pointer is for device-to-user memory transfer whereas
//! DMA is for device-to-device transfer, e.g. directly uploading a captured frame into GPU
//! memory.
//!
//! As you can see, user pointer and DMA are potential candidates for zero-copy applications where
//! buffers should be writable. If a read-only buffer is good enough, MMAP buffers are fine and
//! do not incur any copy overhead either. Most (if not all) devices reporting streaming I/O
//! capabilites support MMAP buffer sharing, but not all support user pointer access.
//!
//! The regular user of this crate will mainly be interested in frame capturing.
//! Here is a very brief example of streaming I/O with memory mapped buffers:
//!
//! ```no_run
//! use v4l::{Buffer, CaptureDevice, MappedBufferStream};
//!
//! let mut dev = CaptureDevice::new(0)
//!     .expect("Failed to open device")
//!     .format(640, 480, b"YUYV")
//!     .expect("Failed to set format")
//!     .fps(30)
//!     .expect("Failed to set frame interval");
//!
//! let stream =
//!     MappedBufferStream::with_buffers(&mut dev, 4).expect("Failed to create buffer stream");
//!
//! for frame in stream {
//!     println!(
//!         "Buffer size: {}, seq: {}, timestamp: {}",
//!        frame.len(),
//!        frame.seq(),
//!        frame.timestamp()
//!    );
//!}
//!```
//!
//! Have a look at the examples to learn more about device and buffer management.

#[cfg(feature = "v4l-sys")]
pub use v4l_sys;

#[cfg(feature = "v4l2-sys")]
pub use v4l2_sys as v4l_sys;

pub mod v4l2;

pub mod ioctl;

pub mod buffer;
pub use buffer::{Buffer, BufferFlags, BufferManager, BufferStream};

pub mod buffers;
pub use buffers::{MappedBuffer, MappedBufferStream};
pub use buffers::{UserBuffer, UserBufferStream};

mod capability;
pub use capability::Capabilities;

mod device;
pub use device::capture;
pub use device::capture_format;
pub use device::capture_parameters;
pub use device::{CaptureDevice, CaptureFormat, CaptureParams};
pub use device::{DeviceInfo, DeviceList};

mod fourcc;
pub use fourcc::FourCC;

mod format;
pub use format::{FormatDescription, FormatFlags};

mod fraction;
pub use fraction::Fraction;

mod memory;
pub use memory::Memory;

mod timestamp;
pub use timestamp::Timestamp;
