use std::{io, marker, mem, os, ptr, slice};

use crate::v4l_sys::*;
use crate::{buffer, v4l2};
use crate::{BufferManager, CaptureDevice, MappedBuffer, Memory};

/// Manage mapped buffers
///
/// All buffers are unmapped in the Drop impl.
/// In case of errors during unmapping, we panic because there is memory corruption going on.
pub struct MappedBufferManager<'a> {
    fd: os::raw::c_int,

    bufs: Vec<(*mut os::raw::c_void, usize)>,
    buf_index: usize,

    phantom: marker::PhantomData<&'a ()>,
}

impl<'a> MappedBufferManager<'a> {
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
    /// use v4l::CaptureDevice;
    /// use v4l::buffers::MappedBufferManager;
    ///
    /// let mut dev = CaptureDevice::new(0);
    /// if let Ok(mut dev) = dev {
    ///     let mgr = MappedBufferManager::new(&mut dev);
    /// }
    /// ```
    pub fn new(dev: &'a mut CaptureDevice) -> Self {
        MappedBufferManager {
            fd: dev.fd(),
            bufs: Vec::new(),
            buf_index: 0,
            phantom: marker::PhantomData,
        }
    }

    pub(crate) fn with_fd(fd: os::raw::c_int) -> Self {
        MappedBufferManager {
            fd,
            bufs: Vec::new(),
            buf_index: 0,
            phantom: marker::PhantomData,
        }
    }
}

impl<'a> Drop for MappedBufferManager<'a> {
    fn drop(&mut self) {
        self.release().unwrap();
    }
}

impl<'a> BufferManager for MappedBufferManager<'a> {
    type Buffer = MappedBuffer<'a>;

    fn allocate(&mut self, count: u32) -> io::Result<u32> {
        let mut v4l2_reqbufs: v4l2_requestbuffers;
        unsafe {
            v4l2_reqbufs = mem::zeroed();
            v4l2_reqbufs.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2_reqbufs.count = count;
            v4l2_reqbufs.memory = Memory::Mmap as u32;
            v4l2::ioctl(
                self.fd,
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
                    self.fd,
                    v4l2::vidioc::VIDIOC_QUERYBUF,
                    &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
                )?;

                v4l2::ioctl(
                    self.fd,
                    v4l2::vidioc::VIDIOC_QBUF,
                    &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
                )?;

                let ptr = v4l2::mmap(
                    ptr::null_mut(),
                    v4l2_buf.length as usize,
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_SHARED,
                    self.fd,
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
                self.fd,
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
                self.fd,
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

        Ok(MappedBuffer::new(
            view,
            buffer::Metadata::new(
                v4l2_buf.sequence,
                v4l2_buf.timestamp.into(),
                v4l2_buf.flags.into(),
            ),
        ))
    }
}
