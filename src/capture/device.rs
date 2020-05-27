use std::convert::TryFrom;
use std::{io, mem, path::Path};

use crate::capture::{Format, Parameters};
use crate::device;
use crate::v4l2;
use crate::v4l_sys::*;
use crate::{DeviceInfo, FormatDescription, FourCC, Fraction, FrameInterval, FrameSize};

/// Linux capture device abstraction
pub struct Device {
    /// raw OS file descriptor
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
        let fd = v4l2::open(path, libc::O_RDWR)?;

        if fd == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(Device { fd })
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    /// Builder: set the format with commonly used parameters
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `fourcc` - Four character code
    ///
    /// # Example
    ///
    /// ```
    /// use std::mem;
    /// use v4l::capture::Device;
    ///
    /// // ignore this, it is necessary to avoid CI failure
    /// if let Ok(dev) = Device::new(0) {
    ///     std::mem::drop(dev);
    ///
    ///     // this is the real example
    ///     let dev = Device::new(0).unwrap().format(640, 480, b"RGB3");
    /// }
    /// ```
    pub fn format(mut self, width: u32, height: u32, fourcc: &[u8; 4]) -> io::Result<Self> {
        let mut fmt = self.get_format()?;
        fmt.width = width;
        fmt.height = height;
        fmt.fourcc = FourCC::new(fourcc);
        self.set_format(&fmt)?;
        Ok(self)
    }

    /// Builder: set frame interval
    ///
    /// # Arguments
    ///
    /// * `fps` - Frames per second
    ///
    /// # Example
    ///
    /// ```
    /// use std::mem;
    /// use v4l::capture::Device;
    ///
    /// // ignore this, it is necessary to avoid CI failure
    /// if let Ok(dev) = Device::new(0) {
    ///     std::mem::drop(dev);
    ///
    ///     // this is the real example
    ///     let dev = Device::new(0).unwrap().format(640, 480, b"RGB3").unwrap().fps(30);
    /// }
    /// ```
    pub fn fps(mut self, fps: u32) -> io::Result<Self> {
        let mut params = self.get_params()?;
        params.interval = Fraction::new(1, fps);
        self.set_params(&params)?;
        Ok(self)
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
        let fd = v4l2::open(path, libc::O_RDWR)?;

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
                v4l2::vidioc::VIDIOC_ENUM_FMT,
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
    ///     let fmt = dev.get_format();
    ///     if let Ok(fmt) = fmt {
    ///         print!("Active format:\n{}", fmt);
    ///     }
    /// }
    /// ```
    pub fn get_format(&self) -> io::Result<Format> {
        unsafe {
            let mut v4l2_fmt: v4l2_format = mem::zeroed();
            v4l2_fmt.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2::ioctl(
                self.fd,
                v4l2::vidioc::VIDIOC_G_FMT,
                &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
            )?;

            Ok(Format::from(v4l2_fmt.fmt.pix))
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
    pub fn set_format(&mut self, fmt: &Format) -> io::Result<Format> {
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

        self.get_format()
    }

    /// Returns a vector of all frame intervals that the device supports
    /// for the given pixel format and frame size.
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::capture::Device;
    /// use v4l::FourCC;
    ///
    /// if let Ok(dev) = Device::new(0) {
    ///     let frameintervals = dev.enumerate_frameintervals(FourCC::new(b"YUYV"), 640, 480);
    ///     if let Ok(frameintervals) = frameintervals {
    ///         for frameinterval in frameintervals {
    ///             print!("{}", frameinterval);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn enumerate_frameintervals(
        &self,
        fourcc: FourCC,
        width: u32,
        height: u32,
    ) -> io::Result<Vec<FrameInterval>> {
        let mut frameintervals = Vec::new();
        let mut v4l2_struct: v4l2_frmivalenum = unsafe { mem::zeroed() };

        v4l2_struct.index = 0;
        v4l2_struct.pixel_format = fourcc.into();
        v4l2_struct.width = width;
        v4l2_struct.height = height;

        loop {
            let ret = unsafe {
                v4l2::ioctl(
                    self.fd,
                    v4l2::vidioc::VIDIOC_ENUM_FRAMEINTERVALS,
                    &mut v4l2_struct as *mut _ as *mut std::os::raw::c_void,
                )
            };

            if ret.is_err() {
                if v4l2_struct.index == 0 {
                    return Err(ret.err().unwrap());
                } else {
                    return Ok(frameintervals);
                }
            }

            if let Ok(frame_interval) = FrameInterval::try_from(v4l2_struct) {
                frameintervals.push(frame_interval);
            }

            v4l2_struct.index += 1;
        }
    }

    /// Returns a vector of valid framesizes that the device supports for the given pixel format
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::capture::Device;
    /// use v4l::FourCC;
    ///
    /// if let Ok(dev) = Device::new(0) {
    ///     let framesizes = dev.enumerate_framesizes(FourCC::new(b"YUYV"));
    ///     if let Ok(framesizes) = framesizes {
    ///         for framesize in framesizes {
    ///             print!("{}", framesize);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn enumerate_framesizes(&self, fourcc: FourCC) -> io::Result<Vec<FrameSize>> {
        let mut framesizes = Vec::new();
        let mut v4l2_struct: v4l2_frmsizeenum = unsafe { mem::zeroed() };

        v4l2_struct.index = 0;
        v4l2_struct.pixel_format = fourcc.into();

        loop {
            let ret = unsafe {
                v4l2::ioctl(
                    self.fd,
                    v4l2::vidioc::VIDIOC_ENUM_FRAMESIZES,
                    &mut v4l2_struct as *mut _ as *mut std::os::raw::c_void,
                )
            };

            if ret.is_err() {
                if v4l2_struct.index == 0 {
                    return Err(ret.err().unwrap());
                } else {
                    return Ok(framesizes);
                }
            }

            if let Ok(frame_size) = FrameSize::try_from(v4l2_struct) {
                framesizes.push(frame_size);
            }

            v4l2_struct.index += 1;
        }
    }

    /// Returns the parameters currently in use
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::capture::Device;
    ///
    /// if let Ok(dev) = Device::new(0) {
    ///     let params = dev.get_params();
    ///     if let Ok(params) = params {
    ///         print!("Active parameters:\n{}", params);
    ///     }
    /// }
    /// ```
    pub fn get_params(&self) -> io::Result<Parameters> {
        unsafe {
            let mut v4l2_params: v4l2_streamparm = mem::zeroed();
            v4l2_params.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2::ioctl(
                self.fd,
                v4l2::vidioc::VIDIOC_G_PARM,
                &mut v4l2_params as *mut _ as *mut std::os::raw::c_void,
            )?;

            Ok(Parameters::from(v4l2_params.parm.capture))
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
    ///     let params = dev.get_params();
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
    pub fn set_params(&mut self, params: &Parameters) -> io::Result<Parameters> {
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

        self.get_params()
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
    /// use v4l::Device;
    /// use v4l::capture::Device as CaptureDevice;
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

impl From<DeviceInfo> for Device {
    fn from(info: DeviceInfo) -> Self {
        let path = info.path().to_path_buf();
        std::mem::drop(info);

        // The DeviceInfo struct was valid, so there should be no way to construct an invalid
        // Device instance here.
        Device::with_path(&path).unwrap()
    }
}
