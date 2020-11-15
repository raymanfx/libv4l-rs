use std::{io, mem, sync::Arc};

use crate::buffer;
use crate::buffer::{Buffer, Metadata};
use crate::device;
use crate::io::arena::Arena as ArenaTrait;
use crate::io::stream::{Capture, Stream as StreamTrait};
use crate::io::userptr::arena::Arena;
use crate::memory::Memory;
use crate::v4l2;
use crate::v4l_sys::*;

/// Stream of user buffers
///
/// An arena instance is used internally for buffer handling.
pub struct Stream {
    handle: Arc<device::Handle>,
    arena: Arena,
    arena_index: usize,
    arena_len: u32,
    buf_type: buffer::Type,

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
    /// use v4l::io::userptr::Stream;
    ///
    /// let dev = Device::new(0);
    /// if let Ok(dev) = dev {
    ///     let stream = Stream::new(&dev);
    /// }
    /// ```
    pub fn new<T: device::Device>(dev: &T) -> io::Result<Self> {
        Stream::with_buffers(dev, 4)
    }

    pub fn with_buffers<T: device::Device>(dev: &T, count: u32) -> io::Result<Self> {
        let mut arena = Arena::new(dev);
        let count = arena.allocate(count)?;

        Ok(Stream {
            handle: dev.handle(),
            arena,
            arena_index: 0,
            arena_len: count,
            buf_type: dev.typ(),
            active: false,
            // the arena queues up all buffers once during allocation
            queued: true,
        })
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            if let Some(code) = e.raw_os_error() {
                // ENODEV means the file descriptor wrapped in the handle became invalid, most
                // likely because the device was unplugged or the connection (USB, PCI, ..)
                // broke down. Handle this case gracefully by ignoring it.
                if code == 19 {
                    /* ignore */
                    return;
                }
            }

            panic!("{:?}", e)
        }
    }
}

impl StreamTrait for Stream {
    fn start(&mut self) -> io::Result<()> {
        unsafe {
            let mut typ = self.buf_type as u32;
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
            let mut typ = self.buf_type as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_STREAMOFF,
                &mut typ as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        Ok(())
    }
}

impl<'a> Capture<'a> for Stream {
    type Item = Buffer<'a>;

    fn queue(&mut self) -> io::Result<()> {
        if self.queued {
            return Ok(());
        }

        let mut v4l2_buf: v4l2_buffer;
        let buf = unsafe { &mut self.arena.get_unchecked(self.arena_index as usize) };
        unsafe {
            v4l2_buf = mem::zeroed();
            v4l2_buf.type_ = self.buf_type as u32;
            v4l2_buf.memory = Memory::UserPtr as u32;
            v4l2_buf.index = self.arena_index as u32;
            v4l2_buf.m.userptr = buf.as_ptr() as std::os::raw::c_ulong;
            v4l2_buf.length = buf.len() as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_QBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.arena_index = (self.arena_index + 1) % self.arena_len as usize;

        Ok(())
    }

    fn dequeue(&'a mut self) -> io::Result<Self::Item> {
        let mut v4l2_buf: v4l2_buffer;
        unsafe {
            v4l2_buf = mem::zeroed();
            v4l2_buf.type_ = self.buf_type as u32;
            v4l2_buf.memory = Memory::UserPtr as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_DQBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }
        self.queued = false;

        let mut buffer = None;
        for i in 0..self.arena.len() {
            unsafe {
                let buf = self.arena.get_unchecked(i);
                if (buf.as_ptr()) == (v4l2_buf.m.userptr as *const u8) {
                    buffer = Some(buf);
                    break;
                }
            }
        }

        match buffer {
            Some(bytes) => Ok(Buffer {
                bytes,
                meta: Metadata {
                    bytesused: v4l2_buf.bytesused,
                    flags: v4l2_buf.flags.into(),
                    field: v4l2_buf.field,
                    timestamp: v4l2_buf.timestamp.into(),
                    sequence: v4l2_buf.sequence,
                },
            }),
            None => Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to find buffer",
            )),
        }
    }

    fn next(&'a mut self) -> io::Result<Self::Item> {
        if !self.active {
            self.start()?;
        }

        self.queue()?;
        self.dequeue()
    }
}
