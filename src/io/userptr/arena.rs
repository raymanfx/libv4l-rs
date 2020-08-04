use std::{io, mem, sync::Arc};

use crate::io::arena::Arena as ArenaTrait;
use crate::v4l2;
use crate::v4l_sys::*;
use crate::{device, memory::Memory};

/// Manage user allocated buffers
///
/// All buffers are released in the Drop impl.
pub struct Arena {
    handle: Arc<device::Handle>,
    bufs: Vec<Vec<u8>>,
}

impl Arena {
    /// Returns a new buffer manager instance
    ///
    /// You usually do not need to use this directly.
    /// A UserBufferStream creates its own manager instance by default.
    ///
    /// # Arguments
    ///
    /// * `dev` - Capture device ref to get its file descriptor
    pub fn new(dev: &dyn device::Device) -> Self {
        Arena {
            handle: dev.handle(),
            bufs: Vec::new(),
        }
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        self.release().unwrap();
    }
}

impl ArenaTrait for Arena {
    type Buffer = [u8];

    fn allocate(&mut self, count: u32) -> io::Result<u32> {
        // we need to get the maximum buffer size from the format first
        let mut v4l2_fmt: v4l2_format;
        unsafe {
            v4l2_fmt = mem::zeroed();
            v4l2_fmt.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_G_FMT,
                &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        #[cfg(feature = "v4l-sys")]
        eprintln!(
            "\n### WARNING ###\n\
            As of early 2020, libv4l2 still does not support USERPTR buffers!\n\
            You may want to use this crate with the raw v4l2 FFI bindings instead!\n"
        );

        let mut v4l2_reqbufs: v4l2_requestbuffers;
        unsafe {
            v4l2_reqbufs = mem::zeroed();
            v4l2_reqbufs.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2_reqbufs.count = count;
            v4l2_reqbufs.memory = Memory::UserPtr as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_REQBUFS,
                &mut v4l2_reqbufs as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        // allocate the new user buffers
        self.bufs.resize(v4l2_reqbufs.count as usize, Vec::new());
        for i in 0..v4l2_reqbufs.count {
            let buf = &mut self.bufs[i as usize];
            unsafe {
                buf.resize(v4l2_fmt.fmt.pix.sizeimage as usize, 0);
            }

            let mut v4l2_buf: v4l2_buffer;
            unsafe {
                v4l2_buf = mem::zeroed();
                v4l2_buf.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
                v4l2_buf.memory = Memory::UserPtr as u32;
                v4l2_buf.index = i;
                v4l2_buf.m.userptr = buf.as_ptr() as u64;
                v4l2_buf.length = v4l2_fmt.fmt.pix.sizeimage;
                v4l2::ioctl(
                    self.handle.fd(),
                    v4l2::vidioc::VIDIOC_QBUF,
                    &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
                )?;
            }
        }

        Ok(v4l2_reqbufs.count)
    }

    fn release(&mut self) -> io::Result<()> {
        // free all buffers by requesting 0
        let mut v4l2_reqbufs: v4l2_requestbuffers;
        unsafe {
            v4l2_reqbufs = mem::zeroed();
            v4l2_reqbufs.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2_reqbufs.count = 0;
            v4l2_reqbufs.memory = Memory::UserPtr as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_REQBUFS,
                &mut v4l2_reqbufs as *mut _ as *mut std::os::raw::c_void,
            )
        }
    }

    fn buffers(&self) -> Vec<&Self::Buffer> {
        self.bufs.iter().map(|buf| &buf[..]).collect()
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
