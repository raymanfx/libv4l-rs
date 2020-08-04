use std::{io, mem, ptr, slice, sync::Arc};

use crate::buffer::Arena as ArenaTrait;
use crate::v4l2;
use crate::v4l_sys::*;
use crate::{device, memory::Memory};

/// Manage mapped buffers
///
/// All buffers are unmapped in the Drop impl.
/// In case of errors during unmapping, we panic because there is memory corruption going on.
pub struct Arena<'a> {
    handle: Arc<device::Handle>,
    bufs: Vec<&'a [u8]>,
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
        }
    }
}

impl<'a> Drop for Arena<'a> {
    fn drop(&mut self) {
        self.release().unwrap();
    }
}

impl<'a> ArenaTrait for Arena<'a> {
    type Buffer = [u8];

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

                let slice = slice::from_raw_parts::<u8>(ptr as *mut u8, v4l2_buf.length as usize);
                self.bufs.push(slice);
            }
        }

        Ok(v4l2_reqbufs.count)
    }

    fn release(&mut self) -> io::Result<()> {
        for buf in &self.bufs {
            unsafe {
                v4l2::munmap(buf.as_ptr() as *mut core::ffi::c_void, buf.len())?;
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

    fn buffers(&self) -> Vec<&Self::Buffer> {
        self.bufs.iter().copied().collect()
    }

    fn get(&self, index: usize) -> Option<&Self::Buffer> {
        if self.bufs.len() > index {
            Some(&self.bufs[index])
        } else {
            None
        }
    }

    fn get_unchecked(&self, index: usize) -> &Self::Buffer {
        &self.bufs[index]
    }
}
