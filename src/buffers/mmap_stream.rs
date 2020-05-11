use std::io;

use crate::buffers::MappedBufferManager;
use crate::v4l2;
use crate::v4l_sys::*;
use crate::{BufferManager, BufferStream, Device, MappedBuffer};

/// Stream of mapped buffers
///
/// A manager instance is used internally for buffer handling.
pub struct MappedBufferStream<'a> {
    dev: &'a mut dyn Device,
    manager: MappedBufferManager<'a>,

    active: bool,
}

impl<'a> MappedBufferStream<'a> {
    /// Returns a stream for frame capturing
    ///
    /// # Arguments
    ///
    /// * `dev` - Capture device ref to get its file descriptor
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::CaptureDevice;
    /// use v4l::buffers::MappedBufferStream;
    ///
    /// let mut dev = CaptureDevice::new(0);
    /// if let Ok(mut dev) = dev {
    ///     let stream = MappedBufferStream::new(&mut dev);
    /// }
    /// ```
    pub fn new(dev: &'a mut dyn Device) -> io::Result<Self> {
        MappedBufferStream::with_buffers(dev, 4)
    }

    pub fn with_buffers(dev: &'a mut dyn Device, count: u32) -> io::Result<Self> {
        let mut manager = MappedBufferManager::with_fd(dev.fd());

        manager.allocate(count)?;

        Ok(MappedBufferStream {
            dev,
            manager,
            active: false,
        })
    }
}

impl<'a> Drop for MappedBufferStream<'a> {
    fn drop(&mut self) {
        self.stop().unwrap();
    }
}

impl<'a> BufferStream for MappedBufferStream<'a> {
    type Buffer = MappedBuffer<'a>;

    fn active(&self) -> bool {
        self.active
    }

    fn start(&mut self) -> io::Result<()> {
        if self.active {
            return Ok(());
        }

        unsafe {
            let mut typ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2::ioctl(
                self.dev.fd(),
                v4l2::vidioc::VIDIOC_STREAMON,
                &mut typ as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        Ok(())
    }

    fn stop(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }

        unsafe {
            let mut typ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2::ioctl(
                self.dev.fd(),
                v4l2::vidioc::VIDIOC_STREAMOFF,
                &mut typ as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        Ok(())
    }

    fn queue(&mut self) -> io::Result<()> {
        self.manager.queue()
    }

    fn dequeue(&mut self) -> io::Result<Self::Buffer> {
        self.manager.dequeue()
    }
}

impl<'a> Iterator for MappedBufferStream<'a> {
    type Item = MappedBuffer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.active && self.start().is_err() {
            return None;
        }

        let buf = self.dequeue();
        if buf.is_err() {
            return None;
        }

        let res = self.queue();
        if res.is_err() {
            return None;
        }

        Some(buf.unwrap())
    }
}
