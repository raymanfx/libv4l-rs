use std::{fmt, mem};

use crate::field::Field;
use crate::fourcc::FourCC;
use crate::v4l_sys::*;
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone)]
/// Streaming format (single-planar)
pub struct Format {
    /// width in pixels
    pub width: u32,
    /// height in pixels
    pub height: u32,
    /// representation of fields
    pub field: Field,
    /// pixelformat code
    pub fourcc: FourCC,

    /// bytes per line
    pub stride: u32,
    /// maximum number of bytes required to store an image
    pub size: u32,
}

impl Format {
    /// Returns a capture format
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `fourcc` - Four character code (pixelformat)
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::FourCC;
    /// use v4l::capture::Format;
    /// let fmt = Format::new(640, 480, FourCC::new(b"YUYV"));
    /// ```
    pub fn new(width: u32, height: u32, fourcc: FourCC) -> Self {
        Format {
            width,
            height,
            field: Field::Any,
            fourcc,
            stride: 0,
            size: 0,
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "width  : {}", self.width)?;
        writeln!(f, "height : {}", self.height)?;
        writeln!(f, "fourcc : {}", self.fourcc)?;
        writeln!(f, "stride : {}", self.stride)?;
        writeln!(f, "size   : {}", self.size)?;
        Ok(())
    }
}

impl From<v4l2_pix_format> for Format {
    fn from(fmt: v4l2_pix_format) -> Self {
        Format {
            width: fmt.width,
            height: fmt.height,
            // Assume that the given format is valid
            field: Field::try_from(fmt.field).unwrap(),
            fourcc: FourCC::from(fmt.pixelformat),
            stride: fmt.bytesperline,
            size: fmt.sizeimage,
        }
    }
}

impl Into<v4l2_pix_format> for Format {
    fn into(self: Format) -> v4l2_pix_format {
        let mut fmt: v4l2_pix_format;
        unsafe {
            fmt = mem::zeroed();
        }

        fmt.width = self.width;
        fmt.height = self.height;
        fmt.field = self.field as u32;
        fmt.pixelformat = self.fourcc.into();
        fmt.bytesperline = self.stride;
        fmt.sizeimage = self.size;
        fmt
    }
}
