use bitflags::bitflags;
use std::convert::TryFrom;
use std::{fmt, mem, str};

use crate::colorspace::Colorspace;
use crate::fieldorder::FieldOrder;
use crate::fourcc::FourCC;
use crate::v4l_sys::*;
use crate::{Quantization, TransferFunction};

bitflags! {
    #[allow(clippy::unreadable_literal)]
    pub struct Flags : u32 {
        const COMPRESSED            = 0x0001;
        const EMULATED              = 0x0002;
        const CONTINUOUS_BITSTREAM  = 0x0004;
        const DYN_RESOLUTION        = 0x0008;
    }
}

impl From<u32> for Flags {
    fn from(flags: u32) -> Self {
        Flags::from_bits_truncate(flags)
    }
}

impl Into<u32> for Flags {
    fn into(self) -> u32 {
        self.bits()
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug)]
/// Format description as returned by VIDIOC_ENUM_FMT
pub struct Description {
    pub index: u32,
    pub typ: u32,
    pub flags: Flags,
    pub description: String,
    pub fourcc: FourCC,
}

impl fmt::Display for Description {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "index       : {}", self.index)?;
        writeln!(f, "type:       : {}", self.typ)?;
        writeln!(f, "flags:      : {}", self.flags)?;
        writeln!(f, "description : {}", self.description)?;
        writeln!(f, "fourcc      : {}", self.fourcc)?;
        Ok(())
    }
}

impl From<v4l2_fmtdesc> for Description {
    fn from(desc: v4l2_fmtdesc) -> Self {
        Description {
            index: desc.index,
            typ: desc.type_,
            flags: Flags::from(desc.flags),
            description: str::from_utf8(&desc.description)
                .unwrap()
                .trim_matches(char::from(0))
                .to_string(),
            fourcc: FourCC::from(desc.pixelformat),
        }
    }
}

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
    /// premultiply alpha; ie. if the alpha channel is 50%, multiply all other channels by 50%
    pub premultiply_alpha: bool,
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
    /// use v4l::format::Format;
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
            premultiply_alpha: false,
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
        writeln!(f, "premultiply alpha : {}", self.premultiply_alpha)?;
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
            premultiply_alpha: (fmt.flags & 0x1) == 0x1,
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
        fmt.flags = if self.premultiply_alpha { 0x1 } else { 0x0 };
        fmt.quantization = self.quantization as u32;
        fmt.xfer_func = self.transfer_function as u32;
        fmt
    }
}
