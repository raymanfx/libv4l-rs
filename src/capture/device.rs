use std::convert::TryFrom;
use std::{io, mem, path::Path};

use crate::v4l2;
use crate::v4l_sys::*;
use crate::{capture, control::Control, device, format};

/// Linux capture device abstraction
pub struct Device {
    /// Raw OS file descriptor
    fd: std::os::raw::c_int,
}

impl Device {
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
    /// use v4l::capture::Device;
    /// let dev = Device::new(0);
    /// ```
    pub fn new(index: usize) -> io::Result<Self> {
        let path = format!("{}{}", "/dev/video", index);
        let fd = v4l2::open(&path, libc::O_RDWR)?;

        if fd == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(Device { fd })
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
    /// use v4l::capture::Device;
    /// let dev = Device::with_path("/dev/video0");
    /// ```
    pub fn with_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let fd = v4l2::open(&path, libc::O_RDWR)?;

        if fd == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(Device { fd })
    }

    /// Returns a vector of valid formats for this device
    ///
    /// The "emulated" field describes formats filled in by libv4lconvert.
    /// There may be a conversion related performance penalty when using them.
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::capture::Device;
    ///
    /// if let Ok(dev) = Device::new(0) {
    ///     let formats = dev.enum_formats();
    ///     if let Ok(formats) = formats {
    ///         for fmt in formats {
    ///             print!("{}", fmt);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn enum_formats(&self) -> io::Result<Vec<format::Description>> {
        let mut formats: Vec<format::Description> = Vec::new();
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
                v4l2::vidioc::VIDIOC_ENUM_FMT,
                &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
            );
        }

        if ret.is_err() {
            return Err(ret.err().unwrap());
        }

        while ret.is_ok() {
            formats.push(format::Description::from(v4l2_fmt));
            v4l2_fmt.index += 1;

            unsafe {
                v4l2_fmt.description = mem::zeroed();
            }

            unsafe {
                ret = v4l2::ioctl(
                    self.fd,
                    v4l2::vidioc::VIDIOC_ENUM_FMT,
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
    /// use v4l::capture::Device;
    ///
    /// if let Ok(dev) = Device::new(0) {
    ///     let fmt = dev.format();
    ///     if let Ok(fmt) = fmt {
    ///         print!("Active format:\n{}", fmt);
    ///     }
    /// }
    /// ```
    pub fn format(&self) -> io::Result<capture::Format> {
        unsafe {
            let mut v4l2_fmt: v4l2_format = mem::zeroed();
            v4l2_fmt.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2::ioctl(
                self.fd,
                v4l2::vidioc::VIDIOC_G_FMT,
                &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
            )?;

            Ok(capture::Format::from(v4l2_fmt.fmt.pix))
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
    /// use v4l::capture::Device;
    ///
    /// if let Ok(mut dev) = Device::new(0) {
    ///     let fmt = dev.format();
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
    pub fn set_format(&mut self, fmt: &capture::Format) -> io::Result<capture::Format> {
        unsafe {
            let mut v4l2_fmt: v4l2_format = mem::zeroed();
            v4l2_fmt.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2_fmt.fmt.pix = (*fmt).into();
            v4l2::ioctl(
                self.fd,
                v4l2::vidioc::VIDIOC_S_FMT,
                &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.format()
    }

    /// Returns the parameters currently in use
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::capture::Device;
    ///
    /// if let Ok(dev) = Device::new(0) {
    ///     let params = dev.params();
    ///     if let Ok(params) = params {
    ///         print!("Active parameters:\n{}", params);
    ///     }
    /// }
    /// ```
    pub fn params(&self) -> io::Result<capture::Parameters> {
        unsafe {
            let mut v4l2_params: v4l2_streamparm = mem::zeroed();
            v4l2_params.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2::ioctl(
                self.fd,
                v4l2::vidioc::VIDIOC_G_PARM,
                &mut v4l2_params as *mut _ as *mut std::os::raw::c_void,
            )?;

            Ok(capture::Parameters::from(v4l2_params.parm.capture))
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
    /// use v4l::capture::{Device, Parameters};
    ///
    /// if let Ok(mut dev) = Device::new(0) {
    ///     let params = dev.params();
    ///     if let Ok(mut params) = params {
    ///         params = Parameters::with_fps(30);
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
    pub fn set_params(&mut self, params: &capture::Parameters) -> io::Result<capture::Parameters> {
        unsafe {
            let mut v4l2_params: v4l2_streamparm = mem::zeroed();
            v4l2_params.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2_params.parm.capture = (*params).into();
            v4l2::ioctl(
                self.fd,
                v4l2::vidioc::VIDIOC_S_PARM,
                &mut v4l2_params as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.params()
    }

    /// Returns the control value for an ID
    ///
    /// # Arguments
    ///
    /// * `id` - Control identifier
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::capture::Device;
    /// use v4l::Control;
    /// use v4l2_sys::V4L2_CID_BRIGHTNESS;
    ///
    /// if let Ok(dev) = Device::new(0) {
    ///     let ctrl = dev.control(V4L2_CID_BRIGHTNESS);
    ///     if let Ok(val) = ctrl {
    ///         match val {
    ///             Control::Value(val) => { println!("Brightness: {}", val) }
    ///             _ => {}
    ///         }
    ///     }
    /// }
    /// ```
    pub fn control(&self, id: u32) -> io::Result<Control> {
        unsafe {
            let mut v4l2_ctrl: v4l2_control = mem::zeroed();
            v4l2_ctrl.id = id;
            v4l2::ioctl(
                self.fd,
                v4l2::vidioc::VIDIOC_G_CTRL,
                &mut v4l2_ctrl as *mut _ as *mut std::os::raw::c_void,
            )?;

            Ok(Control::Value(v4l2_ctrl.value))
        }
    }

    /// Modifies the control value
    ///
    ///
    /// # Arguments
    ///
    /// * `id` - Control identifier
    /// * `val` - New value
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::capture::Device;
    /// use v4l::Control;
    /// use v4l2_sys::V4L2_CID_BRIGHTNESS;
    ///
    /// if let Ok(mut dev) = Device::new(0) {
    ///     dev.set_control(V4L2_CID_BRIGHTNESS, Control::Value(0))
    ///         .expect("Failed to set brightness");
    /// }
    /// ```
    pub fn set_control(&mut self, id: u32, val: Control) -> io::Result<()> {
        unsafe {
            let mut v4l2_ctrl: v4l2_control = mem::zeroed();
            v4l2_ctrl.id = id;
            match val {
                Control::Value(val) => v4l2_ctrl.value = val,
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "only single value controls are supported at the moment",
                    ))
                }
            }
            v4l2::ioctl(
                self.fd,
                v4l2::vidioc::VIDIOC_S_CTRL,
                &mut v4l2_ctrl as *mut _ as *mut std::os::raw::c_void,
            )
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        v4l2::close(self.fd).unwrap();
    }
}

impl device::Device for Device {
    /// Returns the raw fd of the device
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::capture::Device as CaptureDevice;
    /// use v4l::device::Device;
    ///
    /// if let Ok(dev) = CaptureDevice::new(0) {
    ///     print!("Device file descriptor: {}", dev.fd());
    /// }
    /// ```
    fn fd(&self) -> std::os::raw::c_int {
        self.fd
    }

    fn typ(&self) -> device::Type {
        device::Type::VideoCapture
    }
}

impl io::Read for Device {
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

impl TryFrom<device::Info> for Device {
    type Error = io::Error;

    fn try_from(info: device::Info) -> Result<Self, Self::Error> {
        Device::with_path(info.path())
    }
}
