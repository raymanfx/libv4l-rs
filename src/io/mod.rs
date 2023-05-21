use std::{
    io,
    marker::PhantomData,
    mem,
    ops::{Index, IndexMut},
    ptr, slice,
    sync::Arc,
};

use crate::{
    buffer::{Metadata, Type},
    device::Handle,
    memory::{Memory, Mmap, UserPtr},
    v4l2,
    v4l_sys::*,
};

pub mod traits;

pub mod mmap;
pub mod userptr;

/// Manage mapped buffers
///
/// All buffers are unmapped in the Drop impl.
/// In case of errors during unmapping, we panic because there is memory corruption going on.
pub struct Queue<B, S> {
    /// Device handle
    handle: Arc<Handle>,
    /// Type of the buffers
    buf_type: Type,
    /// I/O buffers
    bufs: Vec<B>,
    /// Used to encode state transitions in the type system
    _marker: PhantomData<S>,
}

mod queue {
    pub struct Idle {}

    pub struct Streaming {}
}

impl<B, S> Queue<B, S> {
    /// Returns the number of elements in the queue
    pub fn len(&self) -> usize {
        self.bufs.len()
    }

    /// Request a number of buffers be allocated in the drivers' queue
    ///
    /// Returns the number of actual buffers. Some drivers will require a certain minimum number
    /// of buffers while others might be fine with just a single one.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of buffers to request
    fn reqbufs(&mut self, memory: Memory, count: u32) -> io::Result<u32> {
        let mut v4l2_reqbufs = v4l2_requestbuffers {
            type_: self.buf_type as u32,
            memory: memory as u32,
            count,
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

    /// Query the state of a buffer in the pool
    ///
    /// # Arguments
    ///
    /// * `i` - Buffer index
    pub fn query_buf(&mut self, i: u32) -> io::Result<Metadata> {
        let mut buf = v4l2_buffer {
            index: i,
            type_: self.buf_type as u32,
            ..unsafe { mem::zeroed() }
        };

        unsafe {
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_QUERYBUF,
                &mut buf as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        Ok(Metadata::from(buf))
    }

    /// Start the stream
    fn streamon(&mut self) -> io::Result<()> {
        unsafe {
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_STREAMON,
                &mut self.buf_type as *mut _ as *mut std::os::raw::c_void,
            )
        }
    }

    /// Stop the stream
    fn streamoff(&mut self) -> io::Result<()> {
        unsafe {
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_STREAMOFF,
                &mut self.buf_type as *mut _ as *mut std::os::raw::c_void,
            )
        }
    }

    /// Insert a buffer into the drivers' incoming queue
    fn qbuf(&mut self, buf: &mut v4l2_buffer) -> io::Result<()> {
        buf.type_ = self.buf_type as u32;

        unsafe {
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_QBUF,
                buf as *mut _ as *mut std::os::raw::c_void,
            )
        }
    }

    /// Remove a buffer from the drivers' outgoing queue
    fn dqbuf(&mut self, buf: &mut v4l2_buffer) -> io::Result<()> {
        buf.type_ = self.buf_type as u32;

        unsafe {
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_DQBUF,
                buf as *mut _ as *mut std::os::raw::c_void,
            )
        }
    }
}

impl<B, S> Index<usize> for Queue<B, S> {
    type Output = B;

    fn index(&self, index: usize) -> &Self::Output {
        &self.bufs[index]
    }
}

impl<B, S> IndexMut<usize> for Queue<B, S> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.bufs[index]
    }
}

impl<B> Queue<B, queue::Idle> {
    /// Start the stream
    pub fn start_stream(mut self) -> io::Result<Queue<B, queue::Streaming>> {
        self.streamon()?;

        Ok(Queue {
            handle: self.handle,
            buf_type: self.buf_type,
            bufs: self.bufs,
            _marker: PhantomData,
        })
    }
}

impl<B> Queue<B, queue::Streaming> {
    /// Stop the stream
    pub fn stop_stream(mut self) -> io::Result<Queue<B, queue::Idle>> {
        self.streamoff()?;

        Ok(Queue {
            handle: self.handle,
            buf_type: self.buf_type,
            bufs: self.bufs,
            _marker: PhantomData,
        })
    }
}

impl<S> Queue<Mmap<'_>, S> {
    /// Insert a buffer into the drivers' incoming queue
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer metadata
    pub fn enqueue(&mut self, buf: &Metadata) -> io::Result<()> {
        let mut buf: v4l2_buffer = (*buf).into();
        buf.memory = Memory::Mmap as u32;

        self.qbuf(&mut buf)
    }

    /// Remove a buffer from the drivers' outgoing queue
    pub fn dequeue(&mut self) -> io::Result<Metadata> {
        let mut buf = v4l2_buffer {
            memory: Memory::Mmap as u32,
            ..unsafe { mem::zeroed() }
        };

        self.dqbuf(&mut buf)?;
        Ok(Metadata::from(buf))
    }
}

impl Queue<Mmap<'_>, queue::Idle> {
    /// Request a number of buffers be allocated in the drivers' queue
    ///
    /// Some drivers will require a certain minimum number of buffers while others might be fine
    /// with just a single one.
    ///
    /// # Arguments
    ///
    /// * `handle` - Device handle
    /// * `buf_type` - Buffer type
    /// * `buf_count` - Number of buffers
    pub fn with_mmap(handle: Arc<Handle>, buf_type: Type, buf_count: u32) -> io::Result<Self> {
        let mut queue = Queue {
            handle,
            buf_type,
            bufs: Vec::new(),
            _marker: PhantomData,
        };

        // map new buffers
        let buf_count = queue.reqbufs(Memory::Mmap, buf_count)?;
        for i in 0..buf_count {
            let mut v4l2_buf = v4l2_buffer {
                index: i,
                type_: buf_type as u32,
                memory: Memory::Mmap as u32,
                ..unsafe { mem::zeroed() }
            };

            let mapping = unsafe {
                v4l2::ioctl(
                    queue.handle.fd(),
                    v4l2::vidioc::VIDIOC_QUERYBUF,
                    &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
                )?;

                let ptr = v4l2::mmap(
                    ptr::null_mut(),
                    v4l2_buf.length as usize,
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_SHARED,
                    queue.handle.fd(),
                    v4l2_buf.m.offset as libc::off_t,
                )?;

                slice::from_raw_parts_mut::<u8>(ptr as *mut u8, v4l2_buf.length as usize)
            };

            queue.bufs.push(Mmap(mapping));
        }

        Ok(queue)
    }
}

impl<S> Queue<UserPtr, S> {
    /// Insert a buffer into the drivers' incoming queue
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer metadata
    pub fn enqueue(&mut self, buf: &Metadata) -> io::Result<()> {
        let mut buf: v4l2_buffer = (*buf).into();
        buf.memory = Memory::UserPtr as u32;
        buf.m = v4l2_buffer__bindgen_ty_1 {
            userptr: self.bufs[buf.index as usize].as_ptr() as std::os::raw::c_ulong,
        };

        self.qbuf(&mut buf)
    }

    /// Remove a buffer from the drivers' outgoing queue
    pub fn dequeue(&mut self) -> io::Result<Metadata> {
        let mut buf = v4l2_buffer {
            memory: Memory::UserPtr as u32,
            ..unsafe { mem::zeroed() }
        };

        self.dqbuf(&mut buf)?;
        Ok(Metadata::from(buf))
    }
}

impl Queue<UserPtr, queue::Idle> {
    /// Request a number of buffers be allocated in the drivers' queue
    ///
    /// Some drivers will require a certain minimum number of buffers while others might be fine
    /// with just a single one.
    ///
    /// # Arguments
    ///
    /// * `handle` - Device handle
    /// * `buf_type` - Buffer type
    /// * `buf_count` - Number of buffers
    pub fn with_userptr(handle: Arc<Handle>, buf_type: Type, buf_count: u32) -> io::Result<Self> {
        let mut queue = Queue {
            handle,
            buf_type,
            bufs: Vec::new(),
            _marker: PhantomData,
        };

        // determine the buffer size requirements by asking for the current format
        let mut v4l2_fmt = v4l2_format {
            type_: buf_type as u32,
            ..unsafe { mem::zeroed() }
        };
        unsafe {
            v4l2::ioctl(
                queue.handle.fd(),
                v4l2::vidioc::VIDIOC_G_FMT,
                &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        // request a number of buffers from the driver
        let buf_count = queue.reqbufs(Memory::UserPtr, buf_count)?;

        // allocate the new user buffers
        let buf_len = unsafe { v4l2_fmt.fmt.pix.sizeimage } as usize;
        for _ in 0..buf_count {
            queue.bufs.push(UserPtr(vec![0u8; buf_len]));
        }

        Ok(queue)
    }
}
