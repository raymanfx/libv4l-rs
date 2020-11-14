use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use std::{fs, io, mem, sync::Arc};

use crate::v4l2;
use crate::v4l_sys::*;
use crate::{buffer, control, format};
use crate::{
    capability::Capabilities, control::Control, format::Format, format::FourCC,
    frameinterval::FrameInterval, framesize::FrameSize,
};

/// Manage buffers for a device
pub trait Device {
    /// Returns the raw device handle
    fn handle(&self) -> Arc<Handle>;

    /// Type of the device (capture, overlay, output)
    fn typ(&self) -> buffer::Type;
}

/// Device handle for low-level access.
///
/// Acquiring a handle facilitates (possibly mutating) interactions with the device.
pub struct Handle {
    fd: std::os::raw::c_int,
}

impl Handle {
    /// Returns the raw file descriptor
    pub fn fd(&self) -> std::os::raw::c_int {
        self.fd
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        v4l2::close(self.fd).unwrap();
    }
}

impl From<std::os::raw::c_int> for Handle {
    fn from(fd: std::os::raw::c_int) -> Self {
        Handle { fd }
    }
}

/// Query device properties such as supported formats and controls
pub trait DeviceExt {
    /// Returns a vector of all frame intervals that the device supports for the given pixel format
    /// and frame size
    fn enum_frameintervals(
        &self,
        fourcc: FourCC,
        width: u32,
        height: u32,
    ) -> io::Result<Vec<FrameInterval>>;

    /// Returns a vector of valid framesizes that the device supports for the given pixel format
    fn enum_framesizes(&self, fourcc: FourCC) -> io::Result<Vec<FrameSize>>;

    /// Returns video4linux framework defined information such as card, driver, etc.
    fn query_caps(&self) -> io::Result<Capabilities>;

    /// Returns the supported controls for a device such as gain, focus, white balance, etc.
    fn query_controls(&self) -> io::Result<Vec<control::Description>>;

    /// Returns a vector of valid formats for this device
    ///
    /// The "emulated" field describes formats filled in by libv4lconvert.
    /// There may be a conversion related performance penalty when using them.
    fn enum_formats(&self) -> io::Result<Vec<format::Description>>;

    /// Returns the format currently in use
    fn format(&self) -> io::Result<Format>;

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
    fn set_format(&mut self, fmt: &format::Format) -> io::Result<format::Format>;

    /// Returns the control value for an ID
    ///
    /// # Arguments
    ///
    /// * `id` - Control identifier
    fn control(&self, id: u32) -> io::Result<Control>;

    /// Modifies the control value
    ///
    ///
    /// # Arguments
    ///
    /// * `id` - Control identifier
    /// * `val` - New value
    fn set_control(&mut self, id: u32, val: Control) -> io::Result<()>;
}

impl<T: Device> DeviceExt for T {
    fn enum_frameintervals(
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
                    self.handle().fd(),
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

    fn enum_framesizes(&self, fourcc: FourCC) -> io::Result<Vec<FrameSize>> {
        let mut framesizes = Vec::new();
        let mut v4l2_struct: v4l2_frmsizeenum = unsafe { mem::zeroed() };

        v4l2_struct.index = 0;
        v4l2_struct.pixel_format = fourcc.into();

        loop {
            let ret = unsafe {
                v4l2::ioctl(
                    self.handle().fd(),
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

    fn query_caps(&self) -> io::Result<Capabilities> {
        unsafe {
            let mut v4l2_caps: v4l2_capability = mem::zeroed();
            v4l2::ioctl(
                self.handle().fd(),
                v4l2::vidioc::VIDIOC_QUERYCAP,
                &mut v4l2_caps as *mut _ as *mut std::os::raw::c_void,
            )?;

            Ok(Capabilities::from(v4l2_caps))
        }
    }

    fn query_controls(&self) -> io::Result<Vec<control::Description>> {
        let mut controls = Vec::new();
        unsafe {
            let mut v4l2_ctrl: v4l2_queryctrl = mem::zeroed();

            loop {
                v4l2_ctrl.id |= V4L2_CTRL_FLAG_NEXT_CTRL;
                v4l2_ctrl.id |= V4L2_CTRL_FLAG_NEXT_COMPOUND;
                match v4l2::ioctl(
                    self.handle().fd(),
                    v4l2::vidioc::VIDIOC_QUERYCTRL,
                    &mut v4l2_ctrl as *mut _ as *mut std::os::raw::c_void,
                ) {
                    Ok(_) => {
                        // get the basic control information
                        let mut control = control::Description::from(v4l2_ctrl);

                        // if this is a menu control, enumerate its items
                        if control.typ == control::Type::Menu
                            || control.typ == control::Type::IntegerMenu
                        {
                            let mut items = Vec::new();

                            let mut v4l2_menu: v4l2_querymenu = mem::zeroed();
                            v4l2_menu.id = v4l2_ctrl.id;

                            for i in (v4l2_ctrl.minimum..=v4l2_ctrl.maximum)
                                .step_by(v4l2_ctrl.step as usize)
                            {
                                v4l2_menu.index = i as u32;
                                let res = v4l2::ioctl(
                                    self.handle().fd(),
                                    v4l2::vidioc::VIDIOC_QUERYMENU,
                                    &mut v4l2_menu as *mut _ as *mut std::os::raw::c_void,
                                );

                                // BEWARE OF DRAGONS!
                                // The API docs [1] state VIDIOC_QUERYMENU should may return EINVAL
                                // for some indices between minimum and maximum when an item is not
                                // supported by a driver.
                                //
                                // I have no idea why it is advertised in the first place then, but
                                // have seen this happen with a Logitech C920 HD Pro webcam.
                                // In case of errors, let's just skip the offending index.
                                //
                                // [1] https://github.com/torvalds/linux/blob/master/Documentation/userspace-api/media/v4l/vidioc-queryctrl.rst#description
                                if res.is_err() {
                                    continue;
                                }

                                let item =
                                    control::MenuItem::try_from((control.typ, v4l2_menu)).unwrap();
                                items.push((v4l2_menu.index, item));
                            }

                            control.items = Some(items);
                        }

                        controls.push(control);
                    }
                    Err(e) => {
                        if controls.is_empty() || e.kind() != io::ErrorKind::InvalidInput {
                            return Err(e);
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        Ok(controls)
    }

    fn enum_formats(&self) -> io::Result<Vec<format::Description>> {
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
                self.handle().fd(),
                v4l2::vidioc::VIDIOC_ENUM_FMT,
                &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
            );
        }

        if ret.is_err() {
            // Enumerating the fist format (at index 0) failed, so there are no formats available
            // for this device. Just return an empty vec in this case.
            return Ok(Vec::new());
        }

        while ret.is_ok() {
            formats.push(format::Description::from(v4l2_fmt));
            v4l2_fmt.index += 1;

            unsafe {
                v4l2_fmt.description = mem::zeroed();
            }

            unsafe {
                ret = v4l2::ioctl(
                    self.handle().fd(),
                    v4l2::vidioc::VIDIOC_ENUM_FMT,
                    &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
                );
            }
        }

        Ok(formats)
    }

    fn format(&self) -> io::Result<Format> {
        unsafe {
            let mut v4l2_fmt: v4l2_format = mem::zeroed();
            v4l2_fmt.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2::ioctl(
                self.handle().fd(),
                v4l2::vidioc::VIDIOC_G_FMT,
                &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
            )?;

            Ok(Format::from(v4l2_fmt.fmt.pix))
        }
    }

    fn set_format(&mut self, fmt: &format::Format) -> io::Result<format::Format> {
        unsafe {
            let mut v4l2_fmt: v4l2_format = mem::zeroed();
            v4l2_fmt.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
            v4l2_fmt.fmt.pix = (*fmt).into();
            v4l2::ioctl(
                self.handle().fd(),
                v4l2::vidioc::VIDIOC_S_FMT,
                &mut v4l2_fmt as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.format()
    }

    fn control(&self, id: u32) -> io::Result<Control> {
        unsafe {
            let mut v4l2_ctrl: v4l2_control = mem::zeroed();
            v4l2_ctrl.id = id;
            v4l2::ioctl(
                self.handle().fd(),
                v4l2::vidioc::VIDIOC_G_CTRL,
                &mut v4l2_ctrl as *mut _ as *mut std::os::raw::c_void,
            )?;

            Ok(Control::Value(v4l2_ctrl.value))
        }
    }

    fn set_control(&mut self, id: u32, val: Control) -> io::Result<()> {
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
                self.handle().fd(),
                v4l2::vidioc::VIDIOC_S_CTRL,
                &mut v4l2_ctrl as *mut _ as *mut std::os::raw::c_void,
            )
        }
    }
}

/// Represents a video4linux device node
pub struct Node {
    /// Device node path
    path: PathBuf,
}

impl Node {
    /// Returns a device node observer by path
    ///
    /// The device is opened in read only mode.
    ///
    /// # Arguments
    ///
    /// * `path` - Node path (usually a character device or sysfs entry)
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::device::Node;
    /// let node = Node::new("/dev/video0");
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Node {
            path: PathBuf::from(path.as_ref()),
        }
    }

    /// Returns the absolute path of the device node
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the index of the device node
    pub fn index(&self) -> usize {
        let file_name = self.path.file_name().unwrap();

        let mut index_str = String::new();
        for c in file_name
            .to_str()
            .unwrap()
            .chars()
            .rev()
            .collect::<String>()
            .chars()
        {
            if !c.is_digit(10) {
                break;
            }

            index_str.push(c);
        }

        let index = index_str.parse::<usize>();
        index.unwrap()
    }

    /// Returns name of the device by parsing its sysfs entry
    pub fn name(&self) -> Option<String> {
        let index = self.index();
        let path = format!("{}{}{}", "/sys/class/video4linux/video", index, "/name");
        let name = fs::read_to_string(path);
        match name {
            Ok(name) => Some(name.trim().to_string()),
            Err(_) => None,
        }
    }
}
