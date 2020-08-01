use std::{io, marker, mem, os, slice};

use crate::buffer::{Arena as ArenaTrait, Buffer, Metadata};
use crate::v4l2;
use crate::v4l_sys::*;
use crate::{device::Device, memory::Memory};

/// Manage user allocated buffers
///
/// All buffers are released in the Drop impl.
pub struct Arena<'a> {
    fd: os::raw::c_int,

    bufs: Vec<Vec<u8>>,
    buf_index: usize,

    phantom: marker::PhantomData<&'a ()>,
}

impl<'a> Arena<'a> {
    /// Returns a new buffer manager instance
    ///
    /// You usually do not need to use this directly.
    /// A UserBufferStream creates its own manager instance by default.
    ///
    /// # Arguments
    ///
    /// * `dev` - Capture device ref to get its file descriptor
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::capture::Device;
    /// use v4l::io::userptr::Arena;
    ///
    /// let dev = Device::new(0);
    /// if let Ok(dev) = dev {
    ///     let mgr = Arena::new(&dev);
    /// }
    /// ```
    pub fn new(dev: &'a dyn Device) -> Self {
        Arena {
            fd: dev.fd(),
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
        // we need to get the maximum buffer size from the format first
        let mut v4l2_fmt: v4l2_format;
        unsafe {
            v4l2_fmt = mem::zeroed();
            v4l2_fmt.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2::ioctl(
                self.fd,
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
                self.fd,
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
                    self.fd,
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
                self.fd,
                v4l2::vidioc::VIDIOC_REQBUFS,
                &mut v4l2_reqbufs as *mut _ as *mut std::os::raw::c_void,
            )
        }
    }

    fn queue(&mut self) -> io::Result<()> {
        if self.bufs.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "no buffers allocated"));
        }

        let mut v4l2_buf: v4l2_buffer;
        let buf = &mut self.bufs[self.buf_index as usize];
        unsafe {
            v4l2_buf = mem::zeroed();
            v4l2_buf.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2_buf.memory = Memory::UserPtr as u32;
            v4l2_buf.index = self.buf_index as u32;
            v4l2_buf.m.userptr = buf.as_ptr() as u64;
            v4l2_buf.length = buf.len() as u32;
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
            v4l2_buf.memory = Memory::UserPtr as u32;
            v4l2::ioctl(
                self.fd,
                v4l2::vidioc::VIDIOC_DQBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        let mut index: Option<usize> = None;
        for i in 0..self.bufs.len() {
            let buf = &self.bufs[i];
            unsafe {
                if (buf.as_ptr()) == (v4l2_buf.m.userptr as *const u8) {
                    index = Some(i);
                    break;
                }
            }
        }

        if index.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to find buffer",
            ));
        }

        // The borrow checker prevents us from handing out slices to the internal buffer pool
        // (self.bufs), so we work around this limitation by passing slices to the v4l2_buf
        // instance instead, which holds a pointer itself.
        // That pointer just points back to one of the buffers we allocated ourselves (self.bufs),
        // which we ensured by checking for the index earlier.

        let ptr;
        let view;
        unsafe {
            ptr = v4l2_buf.m.userptr as *mut u8;
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
