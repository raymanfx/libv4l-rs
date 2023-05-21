use std::{
    convert::TryFrom,
    fmt,
    ops::{Deref, DerefMut},
};

use crate::v4l2;

/// Memory used for buffer exchange
#[allow(clippy::unreadable_literal)]
#[rustfmt::skip]
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Memory {
    Mmap        = 1,
    UserPtr     = 2,
    Overlay     = 3,
    DmaBuf      = 4,
}

impl TryFrom<u32> for Memory {
    type Error = ();

    fn try_from(repr: u32) -> Result<Self, Self::Error> {
        match repr {
            1 => Ok(Memory::Mmap),
            2 => Ok(Memory::UserPtr),
            3 => Ok(Memory::Overlay),
            4 => Ok(Memory::DmaBuf),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Memory::Mmap => write!(f, "memory-mapped"),
            Memory::UserPtr => write!(f, "user pointer"),
            Memory::Overlay => write!(f, "overlay"),
            Memory::DmaBuf => write!(f, "DMA buffered"),
        }
    }
}

/// Memory-mapped region
///
/// The backing memory is usually located somewhere on the camera hardware itself. It is mapped
/// into RAM so data can be copied. In case of capture devices, the (virtual) memory can be read.
/// In case of output devices, it can be written.
///
/// The destructor automatically unmaps the memory.
pub struct Mmap<'a>(pub &'a mut [u8]);

impl Drop for Mmap<'_> {
    fn drop(&mut self) {
        unsafe {
            // ignore errors
            let _ = v4l2::munmap(self.0.as_mut_ptr() as *mut core::ffi::c_void, self.0.len());
        }
    }
}

impl Deref for Mmap<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl DerefMut for Mmap<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

/// Userspace memory
///
/// This memory type can be used to directly make the camera hardware write its data into the
/// user-provided buffer (which lives in userspace).
pub struct UserPtr(pub Vec<u8>);

impl Deref for UserPtr {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UserPtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
