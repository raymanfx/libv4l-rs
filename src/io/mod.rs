pub mod traits;

pub mod mmap;
pub mod userptr;

use std::{io, mem, ptr, slice, sync::Arc};

use v4l2_sys::{v4l2_buffer, v4l2_format, v4l2_requestbuffers};

use crate::buffer;
use crate::device::Handle;
use crate::memory::{Memory, Mmap, UserPtr};
use crate::v4l2;

/// Manage mapped buffers
///
/// All buffers are unmapped in the Drop impl.
/// In case of errors during unmapping, we panic because there is memory corruption going on.
pub(crate) struct Arena<T> {
    handle: Arc<Handle>,
    pub bufs: Vec<T>,
    pub buf_mem: Memory,
    pub buf_type: buffer::Type,
}

impl<T> Arena<T> {
    fn request(&mut self, count: u32) -> io::Result<u32> {
        // free all buffers by requesting 0
        let mut v4l2_reqbufs = v4l2_requestbuffers {
            count,
            type_: self.buf_type as u32,
            memory: self.buf_mem as u32,
            ..unsafe { mem::zeroed() }
        };
        unsafe {
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_REQBUFS,
                &mut v4l2_reqbufs as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        Ok(v4l2_reqbufs.count)
    }
}

impl<T> Drop for Arena<T> {
    fn drop(&mut self) {
        if self.bufs.is_empty() {
            // nothing to do
            return;
        }

        // free all buffers by requesting 0
        if let Err(e) = self.request(0) {
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

impl<'a> Arena<Mmap<'a>> {
    /// Returns a new buffer manager instance
    ///
    /// You usually do not need to use this directly.
    /// A MappedBufferStream creates its own manager instance by default.
    ///
    /// # Arguments
    ///
    /// * `handle` - Device handle to get its file descriptor
    /// * `buf_type` - Type of the buffers
    pub fn new(handle: Arc<Handle>, buf_type: buffer::Type) -> Self {
        Arena {
            handle,
            bufs: Vec::new(),
            buf_mem: Memory::Mmap,
            buf_type,
        }
    }

    pub fn allocate(&mut self, count: u32) -> io::Result<u32> {
        let count = self.request(count)?;
        for index in 0..count {
            let mut v4l2_buf = v4l2_buffer {
                index,
                type_: self.buf_type as u32,
                memory: self.buf_mem as u32,
                ..unsafe { mem::zeroed() }
            };
            unsafe {
                v4l2::ioctl(
                    self.handle.fd(),
                    v4l2::vidioc::VIDIOC_QUERYBUF,
                    &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
                )?;

                let ptr = v4l2::mmap(
                    ptr::null_mut(),
                    v4l2_buf.length as usize,
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_SHARED,
                    self.handle.fd(),
                    v4l2_buf.m.offset as libc::off_t,
                )?;

                let slice =
                    slice::from_raw_parts_mut::<u8>(ptr as *mut u8, v4l2_buf.length as usize);
                self.bufs.push(Mmap(slice));
            }
        }

        Ok(count)
    }
}

impl Arena<UserPtr> {
    /// Returns a new buffer manager instance
    ///
    /// You usually do not need to use this directly.
    /// A MappedBufferStream creates its own manager instance by default.
    ///
    /// # Arguments
    ///
    /// * `handle` - Device handle to get its file descriptor
    /// * `buf_type` - Type of the buffers
    pub fn new(handle: Arc<Handle>, buf_type: buffer::Type) -> Self {
        Arena {
            handle,
            bufs: Vec::new(),
            buf_mem: Memory::UserPtr,
            buf_type,
        }
    }

    pub fn allocate(&mut self, count: u32) -> io::Result<u32> {
        // we need to get the maximum buffer size from the format first
        let mut v4l2_fmt = v4l2_format {
            type_: self.buf_type as u32,
            ..unsafe { mem::zeroed() }
        };
        unsafe {
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

        // allocate the new user buffers
        let count = self.request(count)?;
        for _ in 0..count {
            let size = unsafe { v4l2_fmt.fmt.pix.sizeimage };
            let buf = vec![0u8; size as usize];
            self.bufs.push(UserPtr(buf));
        }

        Ok(count)
    }
}
