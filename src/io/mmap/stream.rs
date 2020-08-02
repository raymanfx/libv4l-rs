use std::{io, mem, sync::Arc};

use crate::buffer::{Arena as ArenaTrait, Stream as StreamTrait, StreamItem};
use crate::buffer::{Buffer, Metadata};
use crate::device;
use crate::io::mmap::arena::Arena;
use crate::memory::Memory;
use crate::v4l2;
use crate::v4l_sys::*;

/// Stream of mapped buffers
///
/// An arena instance is used internally for buffer handling.
pub struct Stream {
    handle: Arc<device::Handle>,
    arena: Arena,
    arena_index: usize,

    active: bool,
    queued: bool,
}

impl Stream {
    /// Returns a stream for frame capturing
    ///
    /// # Arguments
    ///
    /// * `dev` - Capture device ref to get its file descriptor
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::capture::Device;
    /// use v4l::io::mmap::Stream;
    ///
    /// let dev = Device::new(0);
    /// if let Ok(dev) = dev {
    ///     let stream = Stream::new(&dev);
    /// }
    /// ```
    pub fn new(dev: &dyn device::Device) -> io::Result<Self> {
        Stream::with_buffers(dev, 4)
    }

    pub fn with_buffers(dev: &dyn device::Device, count: u32) -> io::Result<Self> {
        let mut arena = Arena::new(dev);
        arena.allocate(count)?;

        Ok(Stream {
            handle: dev.handle(),
            arena,
            arena_index: 0,
            active: false,
            // the arena queues up all buffers once during allocation
            queued: true,
        })
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        self.stop().unwrap();
    }
}

impl<'a> StreamTrait<'a> for Stream {
    type Item = Buffer<'a>;

    fn start(&mut self) -> io::Result<()> {
        unsafe {
            let mut typ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_STREAMON,
                &mut typ as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        Ok(())
    }

    fn stop(&mut self) -> io::Result<()> {
        unsafe {
            let mut typ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_STREAMOFF,
                &mut typ as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        Ok(())
    }

    fn queue(&mut self) -> io::Result<()> {
        if self.queued {
            return Ok(());
        }

        let mut v4l2_buf: v4l2_buffer;
        unsafe {
            v4l2_buf = mem::zeroed();
            v4l2_buf.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2_buf.memory = Memory::Mmap as u32;
            v4l2_buf.index = self.arena_index as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_QBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.arena_index += 1;
        if self.arena_index == self.arena.buffers().len() {
            self.arena_index = 0;
        }

        Ok(())
    }

    fn dequeue<'b>(&'b mut self) -> io::Result<StreamItem<'b, Self::Item>> {
        let mut v4l2_buf: v4l2_buffer;
        unsafe {
            v4l2_buf = mem::zeroed();
            v4l2_buf.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2_buf.memory = Memory::Mmap as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_DQBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }
        self.queued = false;

        // The Rust compiler thinks we're returning a value (view) which references data owned by
        // the local function. This is actually not the case since the data slice is memory mapped
        // and thus the actual backing memory resides somewhere else (kernel, on-chip, etc).

        let view = self.arena.buffers()[v4l2_buf.index as usize];
        let view = unsafe { mem::transmute(view) };
        let buf = Buffer::new(
            view,
            Metadata::new(
                v4l2_buf.sequence,
                v4l2_buf.timestamp.into(),
                v4l2_buf.flags.into(),
            ),
        );
        Ok(StreamItem::new(buf))
    }

    fn next(&'a mut self) -> io::Result<StreamItem<'a, Self::Item>> {
        if !self.active {
            self.start()?;
        }

        self.queue()?;
        self.dequeue()
    }
}
