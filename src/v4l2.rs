use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::{io, path::Path};

use crate::ioctl;
use crate::v4l_sys::*;

/// A convenience wrapper around v4l2_open.
///
/// Returns the file descriptor on success.
/// In case of errors, the last OS error will be reported, aka errno on Linux.
///
/// # Arguments
///
/// * `path` - Path to the device node
/// * `flags` - Open flags
///
/// # Example
///
/// ```
/// extern crate v4l;
///
/// use v4l::v4l2;
///
/// let fd = v4l2::open("/dev/video0", libc::O_RDWR);
/// ```
pub fn open<P: AsRef<Path>>(path: P, flags: i32) -> io::Result<std::os::raw::c_int> {
    let fd: std::os::raw::c_int;
    let c_path = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();

    unsafe {
        fd = v4l2_open(c_path.as_ptr(), flags);
    }

    if fd == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(fd)
    }
}

/// A convenience wrapper around v4l2_close.
///
/// In case of errors, the last OS error will be reported, aka errno on Linux.
///
/// # Arguments
///
/// * `fd` - File descriptor of a previously opened device
///
/// # Example
///
/// ```
/// extern crate v4l;
///
/// use v4l::v4l2;
///
/// let fd = v4l2::open("/dev/video0", libc::O_RDWR);
/// if let Ok(fd) = fd {
///     v4l2::close(fd).unwrap();
/// }
/// ```
pub fn close(fd: std::os::raw::c_int) -> io::Result<()> {
    let ret: std::os::raw::c_int;
    unsafe {
        ret = v4l2_close(fd);
    }

    if ret == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// A convenience wrapper around v4l2_ioctl.
///
/// In case of errors, the last OS error will be reported, aka errno on Linux.
///
/// # Arguments
///
/// * `fd` - File descriptor
/// * `request` - IO control code (see [codes])
/// * `argp` - Pointer to memory region holding the argument type
///
/// [codes]: v4l::ioctl::codes
///
/// # Safety
///
/// For maximum flexibility, argp must be a raw pointer. Thus, the entire function is unsafe.
///
/// # Example
///
/// ```
/// extern crate v4l;
///
/// use std::mem;
///
/// use v4l::v4l_sys::*;
/// use v4l::v4l2;
/// use v4l::ioctl::codes;
///
/// let fd = v4l2::open("/dev/video0", libc::O_RDWR);
/// let mut v4l2_caps: v4l2_capability;
/// unsafe {
///     v4l2_caps = mem::zeroed();
/// }
///
/// if let Ok(fd) = fd {
///     unsafe {
///         v4l2::ioctl(fd, codes::VIDIOC_QUERYCAP,
///                     &mut v4l2_caps as *mut _ as *mut std::os::raw::c_void);
///     }
/// }
/// ```
pub unsafe fn ioctl(
    fd: std::os::raw::c_int,
    request: ioctl::_IOC_TYPE,
    argp: *mut std::os::raw::c_void,
) -> io::Result<()> {
    let ret = v4l2_ioctl(fd, request, argp);

    if ret == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
