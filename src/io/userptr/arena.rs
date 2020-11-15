use std::{io, mem, sync::Arc};

use crate::buffer;
use crate::io::arena::Arena as ArenaTrait;
use crate::v4l2;
use crate::v4l_sys::*;
use crate::{device, memory::Memory};

/// Manage user allocated buffers
///
/// All buffers are released in the Drop impl.
pub struct Arena {
    handle: Arc<device::Handle>,
    bufs: Vec<Vec<Vec<u8>>>,
    buf_type: buffer::Type,
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
    pub fn new<T: device::Device>(dev: &T) -> Self {
        Arena {
            handle: dev.handle(),
            bufs: Vec::new(),
            buf_type: dev.typ(),
        }
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        if let Err(e) = self.release() {
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

impl ArenaTrait for Arena {
    type Buffer = Vec<Vec<u8>>;

    fn allocate(&mut self, count: u32) -> io::Result<u32> {
        // we need to get the maximum buffer size from the format first
        let mut v4l2_fmt: v4l2_format;
        unsafe {
            v4l2_fmt = mem::zeroed();
            v4l2_fmt.type_ = self.buf_type as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_G_FMT,
                &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        let num_planes = if !self.buf_type.planar() {
            1
        } else {
            // we need to get the number of image planes from the format
            unsafe { v4l2_fmt.fmt.pix_mp.num_planes as usize }
        };

        #[cfg(feature = "v4l-sys")]
        eprintln!(
            "\n### WARNING ###\n\
            As of early 2020, libv4l2 still does not support USERPTR buffers!\n\
            You may want to use this crate with the raw v4l2 FFI bindings instead!\n"
        );

        let mut v4l2_reqbufs: v4l2_requestbuffers;
        unsafe {
            v4l2_reqbufs = mem::zeroed();
            v4l2_reqbufs.type_ = self.buf_type as u32;
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
            // allocate some backing memory for the planes
            let planes = &mut self.bufs[i as usize];
            planes.resize(num_planes, Vec::new());
            for j in 0..planes.len() {
                let plane_size = if num_planes == 1 {
                    unsafe { v4l2_fmt.fmt.pix.sizeimage }
                } else {
                    unsafe { v4l2_fmt.fmt.pix_mp.plane_fmt[j].sizeimage }
                };
                planes[j].resize(plane_size as usize, 0);
            }

            // TODO: account for data_offset in v4l2_plane

            let mut v4l2_buf: v4l2_buffer;
            let mut v4l2_planes: Vec<v4l2_plane> = Vec::new();
            unsafe {
                v4l2_planes.resize(num_planes as usize, mem::zeroed());
                v4l2_buf = mem::zeroed();
                v4l2_buf.type_ = self.buf_type as u32;
                v4l2_buf.memory = Memory::UserPtr as u32;
                v4l2_buf.index = i;

                if num_planes == 1 {
                    // emulate a single memory plane
                    v4l2_buf.length = v4l2_fmt.fmt.pix.sizeimage;
                    v4l2_buf.m.userptr = planes[0].as_ptr() as std::os::raw::c_ulong;
                } else {
                    v4l2_buf.length = num_planes as u32;
                    v4l2_buf.m.planes = v4l2_planes.as_mut_ptr();
                    for j in 0..v4l2_planes.len() {
                        v4l2_planes[j].length = planes[j].len() as u32;
                        v4l2_planes[j].m.userptr = planes[j].as_ptr() as std::os::raw::c_ulong;
                    }
                }

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
            v4l2_reqbufs.type_ = self.buf_type as u32;
            v4l2_reqbufs.count = 0;
            v4l2_reqbufs.memory = Memory::UserPtr as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_REQBUFS,
                &mut v4l2_reqbufs as *mut _ as *mut std::os::raw::c_void,
            )
        }
    }

    fn get(&self, index: usize) -> Option<&Self::Buffer> {
        Some(self.bufs.get(index)?)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Buffer> {
        Some(self.bufs.get_mut(index)?)
    }

    unsafe fn get_unchecked(&self, index: usize) -> &Self::Buffer {
        self.bufs.get_unchecked(index)
    }

    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Self::Buffer {
        self.bufs.get_unchecked_mut(index)
    }

    fn len(&self) -> usize {
        self.bufs.len()
    }
}
