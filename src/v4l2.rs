use std::io;

use crate::ioctl;
use crate::v4l_sys::*;

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
/// # Example
///
/// ```
/// extern crate v4l;
///
/// use std::mem;
/// use std::ffi::{CString, OsString};
/// use std::os::unix::ffi::OsStrExt;
///
/// use v4l::v4l_sys::*;
/// use v4l::v4l2;
/// use v4l::ioctl::codes;
///
/// let path_str = format!("{}{}", "/dev/video", 0);
/// let c_path = CString::new(OsString::from(path_str).as_bytes()).unwrap();
///
/// let fd: std::os::raw::c_int;
/// let mut v4l2_caps: v4l2_capability;
/// unsafe {
///     fd = v4l2_open(c_path.as_ptr(), libc::O_RDWR);
///     v4l2_caps = mem::zeroed();
/// }
///
/// v4l2::ioctl(fd, codes::VIDIOC_QUERYCAP,
///             &mut v4l2_caps as *mut _ as *mut std::os::raw::c_void);
/// ```
pub fn ioctl(
    fd: std::os::raw::c_int,
    request: ioctl::_IOC_TYPE,
    argp: *mut std::os::raw::c_void,
) -> io::Result<()> {
    let ret: std::os::raw::c_int;

    unsafe {
        ret = v4l2_ioctl(fd, request, argp);
    }

    if ret == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
