pub mod traits;

use std::{
    convert::TryInto,
    io, mem,
    ops::{Deref, DerefMut},
    ptr, slice,
    sync::Arc,
    time::Duration,
};

use v4l2_sys::{v4l2_buffer, v4l2_format, v4l2_requestbuffers};

use crate::buffer::{Buffer, Type};
use crate::device::{Device, Handle};
use crate::io::traits::{CaptureStream, OutputStream, Stream as StreamTrait};
use crate::memory::{Memory, Mmap, UserPtr};
use crate::v4l2;

/// Manage memory buffers
pub(crate) struct Arena<T> {
    handle: Arc<Handle>,
    pub bufs: Vec<Buffer<T>>,
    pub buf_mem: Memory,
    pub buf_type: Type,
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
    pub fn new(handle: Arc<Handle>, buf_type: Type) -> Self {
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
                let buf = Buffer::new(Mmap(slice));
                self.bufs.push(buf);
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
    pub fn new(handle: Arc<Handle>, buf_type: Type) -> Self {
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
            let buf = Buffer::new(UserPtr(vec![0u8; size as usize]));
            self.bufs.push(buf);
        }

        Ok(count)
    }
}

/// Stream of buffers
///
/// An arena instance is used internally for buffer handling.
pub struct Stream<T> {
    handle: Arc<Handle>,
    arena: Arena<T>,
    arena_index: usize,
    buf_type: Type,
    timeout: Option<i32>,

    active: bool,
}

impl<'a> Stream<Mmap<'a>> {
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
    /// use v4l::io::Stream;
    /// use v4l::memory::Mmap;
    ///
    /// let dev = Device::new(0);
    /// if let Ok(dev) = dev {
    ///     let stream = Stream::<Mmap>::new(&dev, Type::VideoCapture);
    /// }
    /// ```
    pub fn new(dev: &Device, buf_type: Type) -> io::Result<Self> {
        Self::with_buffers(dev, buf_type, 4)
    }

    pub fn with_buffers(dev: &Device, buf_type: Type, buf_count: u32) -> io::Result<Self> {
        let mut arena = Arena::<Mmap<'a>>::new(dev.handle(), buf_type);
        let _count = arena.allocate(buf_count)?;

        Ok(Stream {
            handle: dev.handle(),
            arena,
            arena_index: 0,
            buf_type,
            active: false,
            timeout: None,
        })
    }
}

impl Stream<UserPtr> {
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
    /// use v4l::io::Stream;
    /// use v4l::memory::UserPtr;
    ///
    /// let dev = Device::new(0);
    /// if let Ok(dev) = dev {
    ///     let stream = Stream::<UserPtr>::new(&dev, Type::VideoCapture);
    /// }
    /// ```
    pub fn new(dev: &Device, buf_type: Type) -> io::Result<Self> {
        Self::with_buffers(dev, buf_type, 4)
    }

    pub fn with_buffers(dev: &Device, buf_type: Type, buf_count: u32) -> io::Result<Self> {
        let mut arena = Arena::<UserPtr>::new(dev.handle(), buf_type);
        let _count = arena.allocate(buf_count)?;

        Ok(Stream {
            handle: dev.handle(),
            arena,
            arena_index: 0,
            buf_type,
            active: false,
            timeout: None,
        })
    }
}

impl<T> Stream<T> {
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

    fn buffer_desc(&self) -> v4l2_buffer {
        v4l2_buffer {
            type_: self.buf_type as u32,
            memory: self.arena.buf_mem as u32,
            ..unsafe { mem::zeroed() }
        }
    }
}

impl<T> Drop for Stream<T> {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

impl<T> StreamTrait for Stream<T> {
    type Item = Buffer<T>;

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

impl<'a, T> CaptureStream<'a> for Stream<T>
where
    T: Deref<Target = [u8]>,
{
    fn queue(&mut self, index: usize) -> io::Result<()> {
        let mut v4l2_buf = v4l2_buffer {
            index: index as u32,
            ..self.buffer_desc()
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
        let mut v4l2_buf = self.buffer_desc();

        if self.handle.poll(libc::POLLIN, self.timeout.unwrap_or(-1))? == 0 {
            // This condition can only happen if there was a timeout.
            // A timeout is only possible if the `timeout` value is non-zero, meaning we should
            // propagate it to the caller.
            return Err(io::Error::new(io::ErrorKind::TimedOut, "VIDIOC_DQBUF"));
        }

        unsafe {
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_DQBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }
        self.arena_index = v4l2_buf.index as usize;

        let buf = &mut self.arena.bufs[self.arena_index];
        buf.bytesused = v4l2_buf.bytesused;
        buf.flags = v4l2_buf.flags.into();
        buf.field = v4l2_buf.field;
        buf.timestamp = v4l2_buf.timestamp.into();
        buf.sequence = v4l2_buf.sequence;

        Ok(self.arena_index)
    }

    fn next(&'a mut self) -> io::Result<&Self::Item> {
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

        // The index used to access the buffer elements is given to us by v4l2, so we assume it
        // will always be valid.
        Ok(&self.arena.bufs[self.arena_index])
    }
}

impl<'a, T> OutputStream<'a> for Stream<T>
where
    T: DerefMut<Target = [u8]>,
{
    fn queue(&mut self, index: usize) -> io::Result<()> {
        let mut v4l2_buf = v4l2_buffer {
            index: index as u32,
            ..self.buffer_desc()
        };
        unsafe {
            // output settings
            //
            // MetaData.bytesused is initialized to 0. For an output device, when bytesused is
            // set to 0 v4l2 will set it to the size of the plane:
            // https://www.kernel.org/doc/html/v4.15/media/uapi/v4l/buffer.html#struct-v4l2-plane
            v4l2_buf.bytesused = self.arena.bufs[index].bytesused;
            v4l2_buf.field = self.arena.bufs[index].field;

            if self
                .handle
                .poll(libc::POLLOUT, self.timeout.unwrap_or(-1))?
                == 0
            {
                // This condition can only happen if there was a timeout.
                // A timeout is only possible if the `timeout` value is non-zero, meaning we should
                // propagate it to the caller.
                return Err(io::Error::new(io::ErrorKind::TimedOut, "VIDIOC_QBUF"));
            }

            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_QBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )
        }
    }

    fn dequeue(&mut self) -> io::Result<usize> {
        let mut v4l2_buf = self.buffer_desc();

        unsafe {
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_DQBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }
        self.arena_index = v4l2_buf.index as usize;

        let buf = &mut self.arena.bufs[self.arena_index];
        buf.bytesused = v4l2_buf.bytesused;
        buf.flags = v4l2_buf.flags.into();
        buf.field = v4l2_buf.field;
        buf.timestamp = v4l2_buf.timestamp.into();
        buf.sequence = v4l2_buf.sequence;

        Ok(self.arena_index)
    }

    fn next(&'a mut self) -> io::Result<&mut Self::Item> {
        let init = !self.active;
        if !self.active {
            self.start()?;
        }

        // Only queue and dequeue once the buffer has been filled at the call site. The initial
        // call to this function from the call site will happen just after the buffers have been
        // allocated, meaning we need to return the empty buffer initially so it can be filled.
        if !init {
            OutputStream::queue(self, self.arena_index)?;
            self.arena_index = OutputStream::dequeue(self)?;
        }

        // The index used to access the buffer elements is given to us by v4l2, so we assume it
        // will always be valid.
        Ok(&mut self.arena.bufs[self.arena_index])
    }
}
