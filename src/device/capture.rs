use std::ffi::{CString, OsString};
use std::os::unix::ffi::OsStrExt;
use std::{io, mem, path::Path};

use crate::v4l_sys::*;
use crate::{ioctl, v4l2};
use crate::{DeviceInfo, FormatDescription};

#[derive(Debug, Default)]
/// Linux capture device abstraction
pub struct CaptureDevice {
    /// raw OS file descriptor
    fd: std::os::raw::c_int,
}

impl CaptureDevice {
    /// Returns a capture device by index
    ///
    /// Devices are usually enumerated by the system.
    /// An index of zero thus represents the first device the system got to know about.
    ///
    /// # Arguments
    ///
    /// * `index` - Index (0: first, 1: second, ..)
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::CaptureDevice;
    /// let dev = CaptureDevice::new(0);
    /// ```
    pub fn new(index: usize) -> io::Result<Self> {
        let path_str = format!("{}{}", "/dev/video", index);
        let c_path = CString::new(OsString::from(path_str).as_bytes()).unwrap();
        let fd: std::os::raw::c_int;

        unsafe {
            fd = v4l2_open(c_path.as_ptr(), libc::O_RDWR);
        }

        if fd == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(CaptureDevice { fd })
    }

    /// Returns a capture device by path
    ///
    /// Linux device nodes are usually found in /dev/videoX or /sys/class/video4linux/videoX.
    ///
    /// # Arguments
    ///
    /// * `path` - Path (e.g. "/dev/video0")
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::CaptureDevice;
    /// let dev = CaptureDevice::with_path("/dev/video0");
    /// ```
    pub fn with_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let c_path = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
        let fd: std::os::raw::c_int;

        unsafe {
            fd = v4l2_open(c_path.as_ptr(), libc::O_RDWR);
        }

        if fd == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(CaptureDevice { fd })
    }

    /// Returns a vector of valid formats for this device
    ///
    /// The "emulated" field describes formats filled in by libv4lconvert.
    /// There may be a conversion related performance penalty when using them.
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::CaptureDevice;
    /// let dev = CaptureDevice::new(0);
    ///
    /// if let Ok(dev) = dev {
    ///     let formats = dev.enumerate_formats();
    ///     if let Ok(formats) = formats {
    ///         for fmt in formats {
    ///             print!("{}", fmt);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn enumerate_formats(&self) -> io::Result<Vec<FormatDescription>> {
        let mut formats: Vec<FormatDescription> = Vec::new();
        let mut v4l2_fmt: v4l2_fmtdesc;

        unsafe {
            v4l2_fmt = mem::zeroed();
        }

        v4l2_fmt.index = 0;
        v4l2_fmt.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;

        let mut ret: io::Result<()>;

        unsafe {
            ret = v4l2::ioctl(
                self.fd,
                ioctl::codes::VIDIOC_ENUM_FMT,
                &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
            );
        }

        if ret.is_err() {
            return Err(ret.err().unwrap());
        }

        while ret.is_ok() {
            formats.push(FormatDescription::from(v4l2_fmt));
            v4l2_fmt.index += 1;

            unsafe {
                v4l2_fmt.description = mem::zeroed();
            }

            unsafe {
                ret = v4l2::ioctl(
                    self.fd,
                    ioctl::codes::VIDIOC_ENUM_FMT,
                    &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
                );
            }
        }

        Ok(formats)
    }
}

impl Drop for CaptureDevice {
    fn drop(&mut self) {
        unsafe {
            v4l2_close(self.fd);
        }
    }
}

impl From<DeviceInfo> for CaptureDevice {
    fn from(info: DeviceInfo) -> Self {
        let path = info.path().to_path_buf();
        std::mem::drop(info);

        // The DeviceInfo struct was valid, so there should be no way to construct an invalid
        // CaptureDevice instance here.
        CaptureDevice::with_path(&path).unwrap()
    }
}
