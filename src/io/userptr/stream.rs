use std::{io, sync::Arc};

use crate::buffer::{Arena as ArenaTrait, Buffer, Stream as StreamTrait};
use crate::device;
use crate::io::userptr::arena::Arena;
use crate::v4l2;
use crate::v4l_sys::*;

/// Stream of user buffers
///
/// An arena instance is used internally for buffer handling.
pub struct Stream {
    handle: Arc<device::Handle>,
    arena: Arena,

    active: bool,
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
    /// use v4l::io::userptr::Stream;
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
            active: false,
        })
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        self.stop().unwrap();
    }
}

impl<'a> StreamTrait<'a> for Stream {
    type Buffer = Buffer<'a>;

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
        self.arena.queue()
    }

    fn dequeue(&mut self) -> io::Result<Self::Buffer> {
        self.arena.dequeue()
    }

    fn next(&mut self) -> io::Result<Self::Buffer> {
        if !self.active {
            self.start()?;
        }

        let buf = self.dequeue()?;
        self.queue()?;

        Ok(buf)
    }
}
