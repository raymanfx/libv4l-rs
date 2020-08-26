use bitflags::bitflags;
use std::{convert::TryFrom, fmt, mem, str};

use crate::colorspace::Colorspace;
use crate::field::FieldOrder;
use crate::fourcc::FourCC;
use crate::quantization::Quantization;
use crate::transfer::TransferFunction;
use crate::v4l_sys::*;

bitflags! {
    #[allow(clippy::unreadable_literal)]
    pub struct DescriptionFlags : u32 {
        const COMPRESSED            = 0x0001;
        const EMULATED              = 0x0002;
        const CONTINUOUS_BITSTREAM  = 0x0004;
        const DYN_RESOLUTION        = 0x0008;
    }
}

impl From<u32> for DescriptionFlags {
    fn from(flags: u32) -> Self {
        DescriptionFlags::from_bits_truncate(flags)
    }
}

impl Into<u32> for DescriptionFlags {
    fn into(self) -> u32 {
        self.bits()
    }
}

impl fmt::Display for DescriptionFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug)]
/// Format description as returned by VIDIOC_ENUM_FMT
pub struct Description {
    pub index: u32,
    pub typ: u32,
    pub flags: DescriptionFlags,
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
            flags: DescriptionFlags::from(desc.flags),
            description: str::from_utf8(&desc.description)
                .unwrap()
                .trim_matches(char::from(0))
                .to_string(),
            fourcc: FourCC::from(desc.pixelformat),
        }
    }
}

bitflags! {
    #[allow(clippy::unreadable_literal)]
    pub struct Flags : u32 {
        const PREMUL_ALPHA  = 0x00000001;
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

#[derive(Debug, Copy, Clone)]
/// Streaming format (single-planar)
pub struct Format {
    /// width in pixels
    pub width: u32,
    /// height in pixels
    pub height: u32,
    /// pixelformat code
    pub fourcc: FourCC,
    /// field order for interlacing
    pub field_order: FieldOrder,

    /// bytes per line
    pub stride: u32,
    /// maximum number of bytes required to store an image
    pub size: u32,

    /// flags set by the application or driver
    pub flags: Flags,

    /// supplements the pixelformat (fourcc) information
    pub colorspace: Colorspace,
    /// the way colors are mapped
    pub quantization: Quantization,
    /// the transfer function for the colorspace
    pub transfer: TransferFunction,
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
    /// use v4l::{Format, FourCC};
    /// let fmt = Format::new(640, 480, FourCC::new(b"YUYV"));
    /// ```
    pub fn new(width: u32, height: u32, fourcc: FourCC) -> Self {
        Format {
            width,
            height,
            fourcc,
            field_order: FieldOrder::Any,
            stride: 0,
            size: 0,
            flags: Flags::from(0),
            colorspace: Colorspace::Default,
            quantization: Quantization::Default,
            transfer: TransferFunction::Default,
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "width          : {}", self.width)?;
        writeln!(f, "height         : {}", self.height)?;
        writeln!(f, "fourcc         : {}", self.fourcc)?;
        writeln!(f, "field          : {}", self.field_order)?;
        writeln!(f, "stride         : {}", self.stride)?;
        writeln!(f, "size           : {}", self.size)?;
        writeln!(f, "colorspace     : {}", self.colorspace)?;
        writeln!(f, "quantization   : {}", self.quantization)?;
        writeln!(f, "transfer       : {}", self.transfer)?;
        Ok(())
    }
}

impl From<v4l2_pix_format> for Format {
    fn from(fmt: v4l2_pix_format) -> Self {
        Format {
            width: fmt.width,
            height: fmt.height,
            fourcc: FourCC::from(fmt.pixelformat),
            field_order: FieldOrder::try_from(fmt.field).expect("Invalid field order"),
            stride: fmt.bytesperline,
            size: fmt.sizeimage,
            flags: Flags::from(fmt.flags),
            colorspace: Colorspace::try_from(fmt.colorspace).expect("Invalid colorspace"),
            quantization: Quantization::try_from(fmt.quantization).expect("Invalid quantization"),
            transfer: TransferFunction::try_from(fmt.xfer_func).expect("Invalid transfer function"),
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
        fmt.pixelformat = self.fourcc.into();
        fmt.field = self.field_order as u32;
        fmt.bytesperline = self.stride;
        fmt.sizeimage = self.size;
        fmt.colorspace = self.colorspace as u32;
        fmt.flags = self.flags.into();
        fmt.quantization = self.quantization as u32;
        fmt.xfer_func = self.transfer as u32;
        fmt
    }
}
