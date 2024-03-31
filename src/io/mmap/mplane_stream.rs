use std::{io, mem, sync::Arc};
use std::time::Duration;
use std::convert::TryInto;

use v4l2_sys::*;
use crate::v4l2;
use crate::{buffer::{Type, Metadata}, device::{Device, Handle}, memory::Memory};
use crate::io::traits::{Stream as StreamTrait, CaptureStream};
use super::arena::Arena;

pub struct MPlaneStream<'a> {
    handle: Arc<Handle>,
    arena: Arena<'a>,
    arena_index: usize,
    buf_type: Type,
    buf_meta: Vec<Metadata>,
    timeout: Option<i32>,
    mplane_count: u32,

    active: bool,
}

impl<'a> MPlaneStream<'a> {
    /// Returns a stream for Multi-Planar frame capturing
    ///
    /// # Arguments
    /// 
    /// * `dev` - Capture device ref to get its file descriptor
    /// * `mplane_count` - Number of planes
    pub fn new(dev: &Device, buf_type: Type, mplane_count: u32) -> io::Result<Self> {
        assert!(mplane_count == 1, "only support mplane count 1 for now");
        Self::with_buffers(dev, buf_type, mplane_count, 4)
    }

    pub fn with_buffers(dev: &Device, buf_type: Type, mplane_count: u32, buf_count: u32) -> io::Result<Self> {
        assert!(mplane_count == 1, "only support mplane count 1 for now");
        let mut arena = Arena::new(dev.handle(), buf_type);
        let count = arena.allocate_mplane(mplane_count, buf_count)?;
        let mut buf_meta = Vec::new();
        buf_meta.resize(count as usize, Metadata::default());

        Ok(MPlaneStream {
            handle: dev.handle(),
            arena,
            arena_index: 0,
            buf_type,
            buf_meta,
            timeout: None,
            mplane_count,
            active: false,
        })
    }

    /// Returns the raw device handle
    pub fn handle(&self) -> Arc<Handle> {
        self.handle.clone()
    }

    /// Sets a timeout of the v4l file handle.
    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout = Some(duration.as_millis().try_into().unwrap());
    }

    /// Clears the timeout of the v4l file handle.
    pub fn clear_timeout(&mut self) {
        self.timeout = None;
    }

    fn buffer_desc(&self, planes: *mut v4l2_plane, mplane_count: u32) -> v4l2_buffer {
        v4l2_buffer {
            type_: self.buf_type as u32,
            memory: Memory::Mmap as u32,
            length: mplane_count,
            m: v4l2_buffer__bindgen_ty_1 { planes },
            ..unsafe { mem::zeroed() }
        }
    }
}

impl<'a> Drop for MPlaneStream<'a> {
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

impl<'a> StreamTrait for MPlaneStream<'a> {
    type Item = [u8];

    fn start(&mut self) -> io::Result<()> {
        unsafe {
            let mut typ = self.buf_type as u32;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_STREAMON,
                &mut typ as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.active = true;
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

        self.active = false;
        Ok(())
    }
}

impl<'a, 'b> CaptureStream<'b> for MPlaneStream<'a> {
    fn queue(&mut self, index: usize) -> io::Result<()> {
        let mut planes = vec![v4l2_plane {..unsafe { mem::zeroed() }}; self.mplane_count as usize];
        let mut v4l2_buf = v4l2_buffer {
            index: index as u32,
            ..self.buffer_desc(planes.as_mut_ptr(), self.mplane_count)
        };
        
        unsafe {
            v4l2::ioctl(
                self.handle.fd(), 
                v4l2::vidioc::VIDIOC_QBUF, 
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        Ok(())
    }

    fn dequeue(&mut self) -> io::Result<usize> {
        let mut planes = vec![v4l2_plane {..unsafe { mem::zeroed() }}; self.mplane_count as usize];
        let mut v4l2_buf = self.buffer_desc(planes.as_mut_ptr(), self.mplane_count);

        unsafe {
            v4l2::ioctl(
                self.handle.fd(), 
                v4l2::vidioc::VIDIOC_DQBUF, 
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }
        self.arena_index = v4l2_buf.index as usize;

        self.buf_meta[self.arena_index] = Metadata {
            bytesused: unsafe { v4l2_buf.m.planes.as_ref().unwrap().bytesused },
            flags: v4l2_buf.flags.into(),
            field: v4l2_buf.field.into(),
            timestamp: v4l2_buf.timestamp.into(),
            sequence: v4l2_buf.sequence,
        };

        Ok(self.arena_index)
    }

    fn next(&'b mut self) -> io::Result<(&Self::Item, &Metadata)> {
        if !self.active {
            // Enqueue all buffers once on stream start
            for index in 0..self.arena.bufs.len() {
                CaptureStream::queue(self, index)?;
            }

            self.start()?;
        } else {
            CaptureStream::queue(self, self.arena_index)?;
        }

        self.arena_index = CaptureStream::dequeue(self)?;

        let bytes = &self.arena.bufs[self.arena_index];
        let meta = &self.buf_meta[self.arena_index];
        Ok((bytes, meta))

    }
}
