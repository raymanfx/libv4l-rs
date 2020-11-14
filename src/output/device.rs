use std::{io, mem, path::Path, sync::Arc};

use crate::device::Device as DeviceTrait;
use crate::v4l_sys::*;
use crate::{buffer, device};
use crate::{output, v4l2};

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
    /// * `path` - Path (e.g. "/dev/video0")
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::output::Device;
    /// let dev = Device::with_path("/dev/video0");
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
            v4l2_params.type_ = self.typ() as u32;
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
            v4l2_params.type_ = self.typ() as u32;
            v4l2_params.parm.output = (*params).into();
            v4l2::ioctl(
                self.handle.fd(),
                v4l2::vidioc::VIDIOC_S_PARM,
                &mut v4l2_params as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.params()
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

    fn typ(&self) -> buffer::Type {
        buffer::Type::VideoOutput
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
