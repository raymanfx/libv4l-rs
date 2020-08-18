use std::convert::TryFrom;
use std::{fmt, mem};

use crate::colorspace::Colorspace;
use crate::fieldorder::FieldOrder;
use crate::fourcc::FourCC;
use crate::v4l_sys::*;
use crate::{Quantization, TransferFunction};

#[derive(Debug, Copy, Clone)]
/// Streaming format (single-planar)
pub struct Format {
    /// width in pixels
    pub width: u32,
    /// height in pixels
    pub height: u32,
    /// order of fields
    pub fieldorder: FieldOrder,
    /// pixelformat code
    pub fourcc: FourCC,
    /// bytes per line
    pub stride: u32,
    /// maximum number of bytes required to store an image
    pub size: u32,
    /// colorspace of the pixels
    pub colorspace: Colorspace,
    /// the quantization for the colorspace
    pub quantization: Quantization,
    /// the transfer function for the colorspace
    pub transfer_function: TransferFunction,
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
            fieldorder: FieldOrder::Any,
            fourcc,
            stride: 0,
            size: 0,
            colorspace: Colorspace::Default,
            quantization: Quantization::Default,
            transfer_function: TransferFunction::Default,
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "width             : {}", self.width)?;
        writeln!(f, "height            : {}", self.height)?;
        writeln!(f, "field             : {}", self.fieldorder)?;
        writeln!(f, "fourcc            : {}", self.fourcc)?;
        writeln!(f, "stride            : {}", self.stride)?;
        writeln!(f, "size              : {}", self.size)?;
        writeln!(f, "colorspace        : {}", self.colorspace)?;
        writeln!(f, "quantization      : {}", self.quantization)?;
        writeln!(f, "transfer function : {}", self.transfer_function)?;
        Ok(())
    }
}

impl From<v4l2_pix_format> for Format {
    fn from(fmt: v4l2_pix_format) -> Self {
        // Assume that the given format is valid
        Format {
            width: fmt.width,
            height: fmt.height,
            fieldorder: FieldOrder::try_from(fmt.field).expect("Invalid field"),
            fourcc: FourCC::from(fmt.pixelformat),
            stride: fmt.bytesperline,
            size: fmt.sizeimage,
            colorspace: Colorspace::try_from(fmt.colorspace).expect("Invalid colorspace"),
            quantization: Quantization::try_from(fmt.quantization).expect("Invalid quantization"),
            transfer_function: TransferFunction::try_from(fmt.xfer_func)
                .expect("Invalid transfer function"),
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
        fmt.field = self.fieldorder as u32;
        fmt.pixelformat = self.fourcc.into();
        fmt.bytesperline = self.stride;
        fmt.sizeimage = self.size;
        fmt.colorspace = self.colorspace as u32;
        fmt.quantization = self.quantization as u32;
        fmt.xfer_func = self.transfer_function as u32;
        fmt
    }
}
