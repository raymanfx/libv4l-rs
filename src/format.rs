use std::{fmt, str};

use crate::v4l_sys::*;
use crate::FourCC;

#[allow(clippy::unreadable_literal)]
#[rustfmt::skip]
#[repr(u32)]
pub enum FormatFlag {
    Compressed          = 0x0001,
    Emulated            = 0x0002,
    ContinuousBitstream = 0x0004,
    DynResolution       = 0x0008,
}

#[derive(Debug)]
pub struct FormatFlags {
    pub flags: u32,
}

impl From<u32> for FormatFlags {
    fn from(flags: u32) -> Self {
        FormatFlags { flags }
    }
}

impl fmt::Display for FormatFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut prefix = "";
        let mut flags = self.flags;

        let mut print_flag = |flag: FormatFlag, info: &str| -> fmt::Result {
            let flag = flag as u32;
            if flags & flag != 0 {
                write!(f, "{}{}", prefix, info)?;
                prefix = ", ";

                // remove from input flags so we can know about flags we do not recognize
                flags &= !flag;
            }
            Ok(())
        };

        print_flag(FormatFlag::Compressed, "compressed")?;
        print_flag(FormatFlag::Emulated, "emulated")?;
        print_flag(FormatFlag::ContinuousBitstream, "continuous-bytestream")?;
        print_flag(FormatFlag::DynResolution, "dyn-resolution")?;

        if flags != 0 {
            write!(f, "{}{}", prefix, flags)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
/// Format description as returned by VIDIOC_ENUM_FMT
pub struct FormatDescription {
    pub index: u32,
    pub typ: u32,
    pub flags: FormatFlags,
    pub description: String,
    pub fourcc: FourCC,
}

impl fmt::Display for FormatDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "index       : {}", self.index)?;
        writeln!(f, "type:       : {}", self.typ)?;
        writeln!(f, "flags:      : {}", self.flags)?;
        writeln!(f, "description : {}", self.description)?;
        writeln!(f, "fourcc      : {}", self.fourcc)?;
        Ok(())
    }
}

impl From<v4l2_fmtdesc> for FormatDescription {
    fn from(desc: v4l2_fmtdesc) -> Self {
        FormatDescription {
            index: desc.index,
            typ: desc.type_,
            flags: FormatFlags::from(desc.flags),
            description: str::from_utf8(&desc.description).unwrap().to_string(),
            fourcc: FourCC::from(desc.pixelformat),
        }
    }
}
