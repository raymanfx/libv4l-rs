use std::{io, mem, path::Path};

use crate::v4l_sys::*;
use crate::{ioctl, v4l2};
use crate::{CaptureFormat, CaptureParams, DeviceInfo, FormatDescription};

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
        let path = format!("{}{}", "/dev/video", index);
        let fd = v4l2::open(path, libc::O_RDWR)?;

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
        let fd = v4l2::open(path, libc::O_RDWR)?;

        if fd == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(CaptureDevice { fd })
    }

    /// Returns the raw fd of the device
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::CaptureDevice;
    /// let mut dev = CaptureDevice::new(0);
    ///
    /// if let Ok(mut dev) = dev {
    ///     print!("Device file descriptor: {}", dev.fd());
    /// }
    /// ```
    pub fn fd(&mut self) -> std::os::raw::c_int {
        self.fd
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

    /// Returns the format currently in use
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::CaptureDevice;
    /// let dev = CaptureDevice::new(0);
    ///
    /// if let Ok(dev) = dev {
    ///     let fmt = dev.get_format();
    ///     if let Ok(fmt) = fmt {
    ///         print!("Active format:\n{}", fmt);
    ///     }
    /// }
    /// ```
    pub fn get_format(&self) -> io::Result<CaptureFormat> {
        unsafe {
            let mut v4l2_fmt: v4l2_format = mem::zeroed();
            v4l2_fmt.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2::ioctl(
                self.fd,
                ioctl::codes::VIDIOC_G_FMT,
                &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
            )?;

            Ok(CaptureFormat::from(v4l2_fmt.fmt.pix))
        }
    }

    /// Modifies the capture format and returns the actual format
    ///
    /// The driver tries to match the format parameters on a best effort basis.
    /// Thus, if the combination of format properties cannot be achieved, the closest possible
    /// settings are used and reported back.
    ///
    ///
    /// # Arguments
    ///
    /// * `fmt` - Desired format
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::CaptureDevice;
    /// let dev = CaptureDevice::new(0);
    ///
    /// if let Ok(mut dev) = dev {
    ///     let fmt = dev.get_format();
    ///     if let Ok(mut fmt) = fmt {
    ///         fmt.width = 640;
    ///         fmt.height = 480;
    ///         print!("Desired format:\n{}", fmt);
    ///
    ///         let fmt = dev.set_format(&fmt);
    ///         match fmt {
    ///             Ok(fmt) => print!("Actual format:\n{}", fmt),
    ///             Err(e) => print!("{}", e),
    ///         }
    ///     }
    /// }
    /// ```
    pub fn set_format(&mut self, fmt: &CaptureFormat) -> io::Result<CaptureFormat> {
        unsafe {
            let mut v4l2_fmt: v4l2_format = mem::zeroed();
            v4l2_fmt.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2_fmt.fmt.pix = (*fmt).into();
            v4l2::ioctl(
                self.fd,
                ioctl::codes::VIDIOC_S_FMT,
                &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.get_format()
    }

    /// Returns the parameters currently in use
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::CaptureDevice;
    /// let dev = CaptureDevice::new(0);
    ///
    /// if let Ok(dev) = dev {
    ///     let params = dev.get_params();
    ///     if let Ok(params) = params {
    ///         print!("Active parameters:\n{}", params);
    ///     }
    /// }
    /// ```
    pub fn get_params(&self) -> io::Result<CaptureParams> {
        unsafe {
            let mut v4l2_params: v4l2_streamparm = mem::zeroed();
            v4l2_params.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2::ioctl(
                self.fd,
                ioctl::codes::VIDIOC_G_PARM,
                &mut v4l2_params as *mut _ as *mut std::os::raw::c_void,
            )?;

            Ok(CaptureParams::from(v4l2_params.parm.capture))
        }
    }

    /// Modifies the capture parameters and returns the actual parameters
    ///
    ///
    /// # Arguments
    ///
    /// * `params` - Desired parameters
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::{CaptureDevice, CaptureParams};
    /// let dev = CaptureDevice::new(0);
    ///
    /// if let Ok(mut dev) = dev {
    ///     let params = dev.get_params();
    ///     if let Ok(mut params) = params {
    ///         params = CaptureParams::with_fps(30);
    ///         print!("Desired parameters:\n{}", params);
    ///
    ///         let params = dev.set_params(&params);
    ///         match params {
    ///             Ok(params) => print!("Actual parameters:\n{}", params),
    ///             Err(e) => print!("{}", e),
    ///         }
    ///     }
    /// }
    /// ```
    pub fn set_params(&mut self, params: &CaptureParams) -> io::Result<CaptureParams> {
        unsafe {
            let mut v4l2_params: v4l2_streamparm = mem::zeroed();
            v4l2_params.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2_params.parm.capture = (*params).into();
            v4l2::ioctl(
                self.fd,
                ioctl::codes::VIDIOC_S_PARM,
                &mut v4l2_params as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.get_params()
    }
}

impl Drop for CaptureDevice {
    fn drop(&mut self) {
        v4l2::close(self.fd).unwrap();
    }
}

impl io::Read for CaptureDevice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let ret = libc::read(
                self.fd,
                buf.as_mut_ptr() as *mut std::os::raw::c_void,
                buf.len(),
            );
            match ret {
                -1 => Err(io::Error::last_os_error()),
                ret => Ok(ret as usize),
            }
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
