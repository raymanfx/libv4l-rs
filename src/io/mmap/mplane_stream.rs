use std::{io, mem, sync::Arc};

use crate::{buffer::Type, device::{Device, Handle}};

pub struct MPlaneStream {
    handle: Arc<Handle>,
    buf_type: Type,
    mplane_count: u32,
}

impl MPlaneStream {
    /// Returns a stream for Multi-Planar frame capturing
    ///
    /// # Arguments
    /// 
    /// * `dev` - Capture device ref to get its file descriptor
    /// * `mplane_count` - Number of planes
    pub fn new(dev: &Device, buf_type: Type, mplane_count: u32) -> io::Result<Self> {
        Self::with_buffers(dev, buf_type, mplane_count, 4)
    }

    pub fn with_buffers(dev: &Device, buf_type: Type, mplane_count: u32, buf_count: u32) -> io::Result<Self> {
        todo!()
    }
}