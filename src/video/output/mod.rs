pub mod parameters;
pub use parameters::Parameters;

use std::{io, mem};

use crate::buffer::Type;
use crate::device::Device;
use crate::format::{Description as FormatDescription, Format, FourCC};
use crate::frameinterval::FrameInterval;
use crate::framesize::FrameSize;
use crate::v4l2;
use crate::v4l_sys::*;
use crate::video::traits::{Output, Video};

impl Output for Device {
    fn enum_frameintervals(
        &self,
        fourcc: FourCC,
        width: u32,
        height: u32,
    ) -> io::Result<Vec<FrameInterval>> {
        <Self as Video>::enum_frameintervals(self, fourcc, width, height)
    }

    fn enum_framesizes(&self, fourcc: FourCC) -> io::Result<Vec<FrameSize>> {
        <Self as Video>::enum_framesizes(self, fourcc)
    }

    fn enum_formats(&self) -> io::Result<Vec<FormatDescription>> {
        <Self as Video>::enum_formats(self, Type::VideoCapture)
    }

    fn format(&self) -> io::Result<Format> {
        <Self as Video>::format(self, Type::VideoCapture)
    }

    fn set_format(&self, fmt: &Format) -> io::Result<Format> {
        <Self as Video>::set_format(self, Type::VideoCapture, fmt)
    }

    fn params(&self) -> io::Result<Parameters> {
        unsafe {
            let mut v4l2_params = v4l2_streamparm {
                type_: Type::VideoOutput as u32,
                ..mem::zeroed()
            };
            v4l2::ioctl(
                self.handle().fd(),
                v4l2::vidioc::VIDIOC_G_PARM,
                &mut v4l2_params as *mut _ as *mut std::os::raw::c_void,
            )?;

            Ok(Parameters::from(v4l2_params.parm.output))
        }
    }

    fn set_params(&self, params: &Parameters) -> io::Result<Parameters> {
        unsafe {
            let mut v4l2_params = v4l2_streamparm {
                type_: Type::VideoOutput as u32,
                parm: v4l2_streamparm__bindgen_ty_1 {
                    output: (*params).into(),
                },
            };
            v4l2::ioctl(
                self.handle().fd(),
                v4l2::vidioc::VIDIOC_S_PARM,
                &mut v4l2_params as *mut _ as *mut std::os::raw::c_void,
            )?;
        }

        self.params()
    }
}
