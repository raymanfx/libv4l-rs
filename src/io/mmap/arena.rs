use std::{io, marker, mem, os, ptr, slice, sync::Arc};

use crate::buffer::{Arena as ArenaTrait, Buffer, Metadata};
use crate::v4l2;
use crate::v4l_sys::*;
use crate::{device, memory::Memory};

/// Manage mapped buffers
///
/// All buffers are unmapped in the Drop impl.
/// In case of errors during unmapping, we panic because there is memory corruption going on.
pub struct Arena<'a> {
    handle: Arc<device::Handle>,

    bufs: Vec<(*mut os::raw::c_void, usize)>,
    buf_index: usize,

    phantom: marker::PhantomData<&'a ()>,
}

impl<'a> Arena<'a> {
    /// Returns a new buffer manager instance
    ///
    /// You usually do not need to use this directly.
    /// A MappedBufferStream creates its own manager instance by default.
    ///
    /// # Arguments
    ///
    /// * `dev` - Capture device ref to get its file descriptor
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::capture::Device;
    /// use v4l::io::mmap::Arena;
    ///
    /// let dev = Device::new(0);
    /// if let Ok(dev) = dev {
    ///     let mgr = Arena::new(&dev);
    /// }
    /// ```
    pub fn new(dev: &dyn device::Device) -> Self {
        Arena {
            handle: dev.handle(),
            bufs: Vec::new(),
            buf_index: 0,
            phantom: marker::PhantomData,
        }
    }
}

impl<'a> Drop for Arena<'a> {
    fn drop(&mut self) {
        self.release().unwrap();
    }
}

impl<'a> ArenaTrait for Arena<'a> {
    type Buffer = Buffer<'a>;

    fn allocate(&mut self, count: u32) -> io::Result<u32> {
        let mut v4l2_reqbufs: v4l2_requestbuffers;
        unsafe {
            v4l2_reqbufs = mem::zeroed();
            v4l2_reqbufs.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2_reqbufs.count = count;
            v4l2_reqbufs.memory = Memory::Mmap as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_REQBUFS,
                &mut v4l2_reqbufs as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        for i in 0..v4l2_reqbufs.count {
            let mut v4l2_buf: v4l2_buffer;
            unsafe {
                v4l2_buf = mem::zeroed();
                v4l2_buf.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
                v4l2_buf.memory = Memory::Mmap as u32;
                v4l2_buf.index = i;
                v4l2::ioctl(
                    self.handle.fd(),
                    v4l2::vidioc::VIDIOC_QUERYBUF,
                    &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
                )?;

                v4l2::ioctl(
                    self.handle.fd(),
                    v4l2::vidioc::VIDIOC_QBUF,
                    &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
                )?;

                let ptr = v4l2::mmap(
                    ptr::null_mut(),
                    v4l2_buf.length as usize,
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_SHARED,
                    self.handle.fd(),
                    v4l2_buf.m.offset as i64,
                )?;

                self.bufs.push((ptr, v4l2_buf.length as usize));
            }
        }

        Ok(v4l2_reqbufs.count)
    }

    fn release(&mut self) -> io::Result<()> {
        for buf in &self.bufs {
            unsafe {
                v4l2::munmap(buf.0, buf.1)?;
            }
        }

        // free all buffers by requesting 0
        let mut v4l2_reqbufs: v4l2_requestbuffers;
        unsafe {
            v4l2_reqbufs = mem::zeroed();
            v4l2_reqbufs.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2_reqbufs.count = 0;
            v4l2_reqbufs.memory = Memory::Mmap as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_REQBUFS,
                &mut v4l2_reqbufs as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.bufs.clear();
        Ok(())
    }

    fn queue(&mut self) -> io::Result<()> {
        if self.bufs.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "no buffers allocated"));
        }

        let mut v4l2_buf: v4l2_buffer;
        unsafe {
            v4l2_buf = mem::zeroed();
            v4l2_buf.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2_buf.memory = Memory::Mmap as u32;
            v4l2_buf.index = self.buf_index as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_QBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.buf_index += 1;
        if self.buf_index == self.bufs.len() {
            self.buf_index = 0;
        }

        Ok(())
    }

    fn dequeue(&mut self) -> io::Result<Self::Buffer> {
        if self.bufs.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "no buffers allocated"));
        }

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

        let ptr;
        let view;
        unsafe {
            ptr = self.bufs[v4l2_buf.index as usize].0 as *mut u8;
            view = slice::from_raw_parts::<u8>(ptr, v4l2_buf.bytesused as usize);
        }

        Ok(Buffer::new(
            view,
            Metadata::new(
                v4l2_buf.sequence,
                v4l2_buf.timestamp.into(),
                v4l2_buf.flags.into(),
            ),
        ))
    }
}
