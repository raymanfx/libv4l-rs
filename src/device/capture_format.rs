use std::{fmt, mem};

use crate::v4l_sys::*;
use crate::FourCC;

#[derive(Debug, Copy, Clone)]
/// Streaming format (single-planar)
pub struct CaptureFormat {
    /// width in pixels
    pub width: u32,
    /// height in pixels
    pub height: u32,
    /// pixelformat code
    pub fourcc: FourCC,

    /// bytes per line
    pub stride: u32,
    /// maximum number of bytes required to store an image
    pub size: u32,
}

impl CaptureFormat {
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
    /// use v4l::CaptureFormat;
    /// let fmt = CaptureFormat::new(640, 480, FourCC::new(b"YUYV"));
    /// ```
    pub fn new(width: u32, height: u32, fourcc: FourCC) -> Self {
        CaptureFormat {
            width,
            height,
            fourcc,
            stride: 0,
            size: 0,
        }
    }
}

impl fmt::Display for CaptureFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "width  : {}", self.width)?;
        writeln!(f, "height : {}", self.height)?;
        writeln!(f, "fourcc : {}", self.fourcc)?;
        writeln!(f, "stride : {}", self.stride)?;
        writeln!(f, "size   : {}", self.size)?;
        Ok(())
    }
}

impl From<v4l2_pix_format> for CaptureFormat {
    fn from(fmt: v4l2_pix_format) -> Self {
        CaptureFormat {
            width: fmt.width,
            height: fmt.height,
            fourcc: FourCC::from(fmt.pixelformat),
            stride: fmt.bytesperline,
            size: fmt.sizeimage,
        }
    }
}

impl Into<v4l2_pix_format> for CaptureFormat {
    fn into(self: CaptureFormat) -> v4l2_pix_format {
        let mut fmt: v4l2_pix_format;
        unsafe {
            fmt = mem::zeroed();
        }

        fmt.width = self.width;
        fmt.height = self.height;
        fmt.pixelformat = self.fourcc.into();
        fmt.bytesperline = self.stride;
        fmt.sizeimage = self.size;
        fmt
    }
}
