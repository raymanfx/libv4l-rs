use std::{io, mem, sync::Arc};

use crate::buffer;
use crate::buffer::{Buffer, Metadata};
use crate::device::{Device, Handle};
use crate::io::arena::Arena as ArenaTrait;
use crate::io::mmap::arena::Arena;
use crate::io::stream::{Capture, Output, Stream as StreamTrait};
use crate::memory::Memory;
use crate::v4l2;
use crate::v4l_sys::*;

/// Stream of mapped buffers
///
/// An arena instance is used internally for buffer handling.
pub struct Stream<'a> {
    handle: Arc<Handle>,
    arena: Arena<'a>,
    arena_index: usize,
    arena_len: u32,
    buf_type: buffer::Type,

    active: bool,
    queued: bool,
}

impl<'a> Stream<'a> {
    /// Returns a stream for frame capturing
    ///
    /// # Arguments
    ///
    /// * `dev` - Capture device ref to get its file descriptor
    /// * `buf_type` - Type of the buffers
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::buffer::Type;
    /// use v4l::device::Device;
    /// use v4l::io::mmap::Stream;
    ///
    /// let dev = Device::new(0);
    /// if let Ok(dev) = dev {
    ///     let stream = Stream::new(&dev, Type::VideoCapture);
    /// }
    /// ```
    pub fn new(dev: &Device, buf_type: buffer::Type) -> io::Result<Self> {
        Stream::with_buffers(dev, buf_type, 4)
    }

    pub fn with_buffers(dev: &Device, buf_type: buffer::Type, buf_count: u32) -> io::Result<Self> {
        let mut arena = Arena::new(dev.handle(), buf_type);
        let count = arena.allocate(buf_count)?;

        Ok(Stream {
            handle: dev.handle(),
            arena,
            arena_index: 0,
            arena_len: count,
            buf_type,
            active: false,
            // the arena queues up all buffers once during allocation
            queued: true,
        })
    }
}

impl<'a> Drop for Stream<'a> {
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

impl<'a> StreamTrait for Stream<'a> {
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

impl<'a, 'b> Capture<'b> for Stream<'a> {
    type Item = Buffer<'b>;

    fn queue(&mut self) -> io::Result<()> {
        if self.queued {
            return Ok(());
        }

        let mut v4l2_buf: v4l2_buffer;
        unsafe {
            v4l2_buf = mem::zeroed();
            v4l2_buf.type_ = self.buf_type as u32;
            v4l2_buf.memory = Memory::Mmap as u32;
            v4l2_buf.index = self.arena_index as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_QBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.arena_index = (self.arena_index + 1) % self.arena_len as usize;

        Ok(())
    }

    fn dequeue(&'b mut self) -> io::Result<Self::Item> {
        let mut v4l2_buf: v4l2_buffer;
        unsafe {
            v4l2_buf = mem::zeroed();
            v4l2_buf.type_ = self.buf_type as u32;
            v4l2_buf.memory = Memory::Mmap as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_DQBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }
        self.queued = false;

        let bytes = unsafe { self.arena.get_unchecked(v4l2_buf.index as usize) };
        Ok(Buffer {
            bytes,
            meta: Metadata {
                bytesused: v4l2_buf.bytesused,
                flags: v4l2_buf.flags.into(),
                field: v4l2_buf.field,
                timestamp: v4l2_buf.timestamp.into(),
                sequence: v4l2_buf.sequence,
            },
        })
    }

    fn next(&'b mut self) -> io::Result<Self::Item> {
        if !self.active {
            self.start()?;
        }

        <Stream as Capture>::queue(self)?;
        <Stream as Capture>::dequeue(self)
    }
}

impl<'a, 'b> Output<'b> for Stream<'a> {
    type Item = Buffer<'b>;

    fn queue(&mut self, item: Self::Item) -> io::Result<()> {
        if self.queued {
            return Ok(());
        }

        let mut v4l2_buf: v4l2_buffer;
        unsafe {
            v4l2_buf = mem::zeroed();
            v4l2_buf.type_ = self.buf_type as u32;
            v4l2_buf.memory = Memory::Mmap as u32;
            v4l2_buf.index = self.arena_index as u32;
            // output settings
            v4l2_buf.bytesused = item.data().len() as u32;
            v4l2_buf.field = item.meta.field;

            // write the actual frame data
            let bytes = self.arena.get_unchecked_mut(v4l2_buf.index as usize);
            bytes.copy_from_slice(item.data());

            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_QBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.arena_index = (self.arena_index + 1) % self.arena_len as usize;

        Ok(())
    }

    fn dequeue(&mut self) -> io::Result<()> {
        let mut v4l2_buf: v4l2_buffer;
        unsafe {
            v4l2_buf = mem::zeroed();
            v4l2_buf.type_ = self.buf_type as u32;
            v4l2_buf.memory = Memory::Mmap as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_DQBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.arena_index = v4l2_buf.index as usize;
        self.queued = false;

        Ok(())
    }

    fn next(&mut self, item: Self::Item) -> io::Result<()> {
        if !self.active {
            self.start()?;
        }

        <Stream as Output>::queue(self, item)?;
        <Stream as Output>::dequeue(self)
    }
}
