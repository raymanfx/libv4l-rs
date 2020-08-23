use std::convert::TryFrom;
use std::{io, mem, path::Path, sync::Arc};

use crate::v4l_sys::*;
use crate::{device, format};
use crate::{output, v4l2, Control};

/// Linux output device abstraction
pub struct Device {
    /// Raw handle
    handle: Arc<device::Handle>,
}

impl Device {
    /// Returns an output device by index
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
    /// use v4l::output::Device;
    /// let dev = Device::new(0);
    /// ```
    pub fn new(index: usize) -> io::Result<Self> {
        let path = format!("{}{}", "/dev/video", index);
        let fd = v4l2::open(&path, libc::O_RDWR)?;

        if fd == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(Device {
            handle: Arc::new(device::Handle::from(fd)),
        })
    }

    /// Returns an output device by path
    ///
    /// Linux device nodes are usually found in /dev/videoX or /sys/class/video4linux/videoX.
    ///
    /// # Arguments
    ///
    /// * `path` - Path (e.g. "/dev/video1")
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::output::Device;
    /// let dev = Device::with_path("/dev/video1");
    /// ```
    pub fn with_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let fd = v4l2::open(&path, libc::O_RDWR)?;

        if fd == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(Device {
            handle: Arc::new(device::Handle::from(fd)),
        })
    }

    /// Returns a vector of valid formats for this device
    ///
    /// The "emulated" field describes formats filled in by libv4lconvert.
    /// There may be a conversion related performance penalty when using them.
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::output::Device;
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
        v4l2_fmt.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_OUTPUT;

        let mut ret: io::Result<()>;

        unsafe {
            ret = v4l2::ioctl(
                self.handle.fd(),
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
                    self.handle.fd(),
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
    /// use v4l::output::Device;
    ///
    /// if let Ok(dev) = Device::new(0) {
    ///     let fmt = dev.format();
    ///     if let Ok(fmt) = fmt {
    ///         print!("Active format:\n{}", fmt);
    ///     }
    /// }
    /// ```
    pub fn format(&self) -> io::Result<format::Format> {
        unsafe {
            let mut v4l2_fmt: v4l2_format = mem::zeroed();
            v4l2_fmt.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_OUTPUT;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_G_FMT,
                &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
            )?;

            Ok(format::Format::from(v4l2_fmt.fmt.pix))
        }
    }

    /// Modifies the output format and returns the actual format
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
    /// use v4l::output::Device;
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
    pub fn set_format(&mut self, fmt: &format::Format) -> io::Result<format::Format> {
        unsafe {
            let mut v4l2_fmt: v4l2_format = mem::zeroed();
            v4l2_fmt.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_OUTPUT;
            v4l2_fmt.fmt.pix = (*fmt).into();
            v4l2::ioctl(
                self.handle.fd(),
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
    /// use v4l::output::Device;
    ///
    /// if let Ok(dev) = Device::new(0) {
    ///     let params = dev.params();
    ///     if let Ok(params) = params {
    ///         print!("Active parameters:\n{}", params);
    ///     }
    /// }
    /// ```
    pub fn params(&self) -> io::Result<output::Parameters> {
        unsafe {
            let mut v4l2_params: v4l2_streamparm = mem::zeroed();
            v4l2_params.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_OUTPUT;
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_G_PARM,
                &mut v4l2_params as *mut _ as *mut std::os::raw::c_void,
            )?;

            Ok(output::Parameters::from(v4l2_params.parm.output))
        }
    }

    /// Modifies the output parameters and returns the actual parameters
    ///
    ///
    /// # Arguments
    ///
    /// * `params` - Desired parameters
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::output::{Device, Parameters};
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
    pub fn set_params(&mut self, params: &output::Parameters) -> io::Result<output::Parameters> {
        unsafe {
            let mut v4l2_params: v4l2_streamparm = mem::zeroed();
            v4l2_params.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_OUTPUT;
            v4l2_params.parm.output = (*params).into();
            v4l2::ioctl(
                self.handle.fd(),
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
    /// use v4l::output::Device;
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
                self.handle.fd(),
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
    /// use v4l::output::Device;
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
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_S_CTRL,
                &mut v4l2_ctrl as *mut _ as *mut std::os::raw::c_void,
            )
        }
    }
}

impl device::Device for Device {
    /// Returns the handle of the device
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::output::Device as OutputDevice;
    /// use v4l::device::Device;
    ///
    /// if let Ok(dev) = OutputDevice::new(0) {
    ///     print!("Device file descriptor: {}", dev.handle().fd());
    /// }
    /// ```
    fn handle(&self) -> Arc<device::Handle> {
        Arc::clone(&self.handle)
    }

    fn typ(&self) -> device::Type {
        device::Type::VideoOutput
    }
}

impl io::Write for Device {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            let ret = libc::write(
                self.handle.fd(),
                buf.as_ptr() as *const std::os::raw::c_void,
                buf.len(),
            );

            match ret {
                -1 => Err(io::Error::last_os_error()),
                ret => Ok(ret as usize),
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        // write doesn't use a buffer, so it effectively flushes with each call
        // therefore, we don't have anything to flush later
        Ok(())
    }
}

impl TryFrom<device::Info> for Device {
    type Error = io::Error;

    fn try_from(info: device::Info) -> Result<Self, Self::Error> {
        Device::with_path(info.path())
    }
}
