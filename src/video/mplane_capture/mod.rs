use crate::{buffer::Type, Device};
use std::{io, mem};

use super::MultiPlanarCapture;
use crate::format::Format;
use crate::v4l2;
use crate::v4l_sys::*;

impl MultiPlanarCapture for Device {
    impl_format!(Type::VideoCaptureMplane);
    impl_set_format!(Type::VideoCaptureMplane);


}