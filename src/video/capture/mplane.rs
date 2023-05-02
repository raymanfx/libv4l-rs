use std::convert::TryFrom;
use std::{io, mem};

use super::Parameters;
use crate::buffer::Type::VideoCaptureMplane;
use crate::device::MultiPlaneDevice;
use crate::format::FourCC;
use crate::format::{Description as FormatDescription, MultiPlaneFormat};
use crate::frameinterval::FrameInterval;
use crate::framesize::FrameSize;
use crate::v4l2;
use crate::v4l_sys::*;
use crate::video::traits::Capture;

impl Capture for MultiPlaneDevice {
    impl_enum_frameintervals!();
    impl_enum_framesizes!();
    impl_format!(VideoCaptureMplane);
    impl_set_format!(VideoCaptureMplane);
    impl_enum_formats!(VideoCaptureMplane);

    type Format = MultiPlaneFormat;

    fn params(&self) -> io::Result<Parameters> {
        unimplemented!()
    }

    fn set_params(&self, _params: &Parameters) -> io::Result<Parameters> {
        unimplemented!()
    }
}
