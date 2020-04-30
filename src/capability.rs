use std::{fmt, str};

use crate::v4l_sys::*;

#[allow(clippy::unreadable_literal)]
#[rustfmt::skip]
#[repr(u32)]
pub enum Capability {
    VideoCapture        = 0x00000001,
    VideoOutput         = 0x00000002,
    VideoOverlay        = 0x00000004,
    VbiCaputre          = 0x00000010,
    VbiOutput           = 0x00000020,
    SlicedVbiCapture    = 0x00000040,
    SlicedVbiOutput     = 0x00000080,
    RdsCapture          = 0x00000100,
    VideoOutputOverlay  = 0x00000200,
    HwFreqSeek          = 0x00000400,
    RdsOutput           = 0x00000800,
    
    VideoCaptureMplane  = 0x00001000,
    VideoOutputMplane   = 0x00002000,
    VideoM2MMplane      = 0x00004000,
    VideoM2M            = 0x00008000,

    Tuner               = 0x00010000,
    Audio               = 0x00020000,
    Radio               = 0x00040000,
    Modulator           = 0x00080000,

    SdrCapture          = 0x00100000,
    ExtPixFormat        = 0x00200000,
    SdrOutput           = 0x00400000,
    MetaCapture         = 0x00800000,

    ReadWrite           = 0x01000000,
    AsyncIO             = 0x02000000,
    Streaming           = 0x04000000,
    MetaOutput          = 0x08000000,

    Touch               = 0x10000000,

    DeviceCaps          = 0x80000000,
}

#[derive(Debug)]
/// Device capability flags
pub struct DeviceCapabilites {
    /// Capability flags such as V4L2_CAP_VIDEO_CAPTURE
    pub flags: u32,
}

impl From<u32> for DeviceCapabilites {
    fn from(flags: u32) -> Self {
        DeviceCapabilites { flags }
    }
}

impl fmt::Display for DeviceCapabilites {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut prefix = "";
        let mut flags = self.flags;

        let mut print_flag = |flag: Capability, info: &str| -> fmt::Result {
            let flag = flag as u32;
            if flags & flag != 0 {
                write!(f, "{}{}", prefix, info)?;
                prefix = ", ";

                // remove from input flags so we can know about flags we do not recognize
                flags &= !flag;
            }
            Ok(())
        };

        print_flag(Capability::VideoCapture, "Video Capture")?;
        print_flag(Capability::VideoCaptureMplane, "Video Capture Multiplanar")?;
        print_flag(Capability::VideoOutput, "Video Output")?;
        print_flag(Capability::VideoOutputMplane, "Video Output Multiplanar")?;
        print_flag(Capability::VideoM2M, "Video Memory-to-Memory")?;
        print_flag(Capability::VideoM2MMplane, "Video Capture")?;
        print_flag(Capability::VideoOverlay, "Video Overlay")?;
        print_flag(Capability::VideoOutputOverlay, "Video Output Overlay")?;
        print_flag(Capability::VbiCaputre, "VBI Capture")?;
        print_flag(Capability::VbiOutput, "VBI Output")?;
        print_flag(Capability::SlicedVbiCapture, "Sliced VBI Capture")?;
        print_flag(Capability::SlicedVbiOutput, "Sliced VBI Output")?;
        print_flag(Capability::RdsCapture, "RDS Capture")?;
        print_flag(Capability::RdsOutput, "RDS Output")?;
        print_flag(Capability::SdrCapture, "SDR Capture")?;
        print_flag(Capability::SdrOutput, "SDR Output")?;
        print_flag(Capability::MetaCapture, "Metadata Capture")?;
        print_flag(Capability::MetaOutput, "Metadata Output")?;
        print_flag(Capability::Tuner, "Tuner")?;
        print_flag(Capability::Touch, "Touch Device")?;
        print_flag(Capability::HwFreqSeek, "HW Frequency Seek")?;
        print_flag(Capability::Modulator, "Modulator")?;
        print_flag(Capability::Audio, "Audio")?;
        print_flag(Capability::Radio, "Radio")?;
        print_flag(Capability::ReadWrite, "Read/Write")?;
        print_flag(Capability::AsyncIO, "Async I/O")?;
        print_flag(Capability::Streaming, "Streaming")?;
        print_flag(Capability::ExtPixFormat, "Extended Pix Format")?;
        print_flag(Capability::DeviceCaps, "Device Capabilities")?;

        if flags != 0 {
            write!(f, "{}{}", prefix, flags)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
/// Device capabilities
pub struct Capabilities {
    /// Driver name, e.g. uvc for usb video class devices
    pub driver: String,
    /// Card name
    pub card: String,
    /// Bus name, e.g. USB or PCI
    pub bus: String,
    /// Version number MAJOR.MINOR.PATCH
    pub version: (u8, u8, u8),

    /// Capability flags
    pub capabilities: DeviceCapabilites,
}

impl From<v4l2_capability> for Capabilities {
    fn from(cap: v4l2_capability) -> Self {
        let mut caps = Capabilities {
            driver: str::from_utf8(&cap.driver).unwrap().to_string(),
            card: str::from_utf8(&cap.card).unwrap().to_string(),
            bus: str::from_utf8(&cap.bus_info).unwrap().to_string(),
            version: (0, 0, 0),
            capabilities: DeviceCapabilites {
                flags: cap.device_caps,
            },
        };

        caps.version.0 = ((cap.version >> 16) & 0xff) as u8;
        caps.version.1 = ((cap.version >> 8) & 0xff) as u8;
        caps.version.2 = (cap.version & 0xff) as u8;
        caps
    }
}

impl fmt::Display for Capabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Driver      : {}", self.driver)?;
        writeln!(f, "Card        : {}", self.card)?;
        writeln!(f, "Bus         : {}", self.bus)?;
        writeln!(
            f,
            "Version     : {}.{}.{}",
            self.version.0, self.version.1, self.version.2
        )?;
        writeln!(f, "Capabilites : {}", self.capabilities)?;
        Ok(())
    }
}
