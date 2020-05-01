use std::{fmt, mem};

use crate::v4l_sys::*;
use crate::Fraction;

#[allow(clippy::unreadable_literal)]
#[rustfmt::skip]
#[repr(u32)]
pub enum Capability {
    TimePerFrame        = 0x1000,
}

#[allow(clippy::unreadable_literal)]
#[rustfmt::skip]
#[repr(u32)]
pub enum Mode {
    HighQuality         = 0x0001,
}

#[derive(Debug, Default, Copy, Clone)]
/// Parameter capability flags
pub struct ParameterCapabilites {
    /// Capability flags such as V4L2_CAP_TIMEPERFRAME
    pub flags: u32,
}

impl From<u32> for ParameterCapabilites {
    fn from(flags: u32) -> Self {
        ParameterCapabilites { flags }
    }
}

impl Into<u32> for ParameterCapabilites {
    fn into(self) -> u32 {
        self.flags
    }
}

impl fmt::Display for ParameterCapabilites {
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

        print_flag(Capability::TimePerFrame, "Time per frame")?;

        if flags != 0 {
            write!(f, "{}{}", prefix, flags)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Copy, Clone)]
/// Parameter mode flags
pub struct ParameterModes {
    /// Mode flags such as V4L2_MODE_HIGHQUALITY
    pub flags: u32,
}

impl From<u32> for ParameterModes {
    fn from(flags: u32) -> Self {
        ParameterModes { flags }
    }
}

impl Into<u32> for ParameterModes {
    fn into(self) -> u32 {
        self.flags
    }
}

impl fmt::Display for ParameterModes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut prefix = "";
        let mut flags = self.flags;

        let mut print_flag = |flag: Mode, info: &str| -> fmt::Result {
            let flag = flag as u32;
            if flags & flag != 0 {
                write!(f, "{}{}", prefix, info)?;
                prefix = ", ";

                // remove from input flags so we can know about flags we do not recognize
                flags &= !flag;
            }
            Ok(())
        };

        print_flag(Mode::HighQuality, "High quality")?;

        if flags != 0 {
            write!(f, "{}{}", prefix, flags)?;
        }
        Ok(())
    }
}

impl From<v4l2_fract> for Fraction {
    fn from(frac: v4l2_fract) -> Self {
        Fraction {
            numerator: frac.numerator,
            denominator: frac.denominator,
        }
    }
}

impl Into<v4l2_fract> for Fraction {
    fn into(self: Fraction) -> v4l2_fract {
        let mut frac: v4l2_fract;
        unsafe {
            frac = mem::zeroed();
        }

        frac.numerator = self.numerator;
        frac.denominator = self.denominator;
        frac
    }
}

#[derive(Debug, Copy, Clone)]
/// Streaming parameters (single-planar)
pub struct CaptureParams {
    pub capabilities: ParameterCapabilites,
    pub modes: ParameterModes,
    pub interval: Fraction,
}

impl CaptureParams {
    pub fn new(frac: Fraction) -> Self {
        CaptureParams {
            capabilities: ParameterCapabilites::default(),
            modes: ParameterModes::default(),
            interval: frac,
        }
    }

    pub fn with_fps(fps: u32) -> Self {
        CaptureParams {
            capabilities: ParameterCapabilites::default(),
            modes: ParameterModes::default(),
            interval: Fraction::new(1, fps),
        }
    }
}

impl fmt::Display for CaptureParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "capabilities : {}", self.capabilities)?;
        writeln!(f, "modes        : {}", self.modes)?;
        writeln!(f, "interval     : {} [s]", self.interval)?;
        Ok(())
    }
}

impl From<v4l2_captureparm> for CaptureParams {
    fn from(params: v4l2_captureparm) -> Self {
        CaptureParams {
            capabilities: ParameterCapabilites::from(params.capability),
            modes: ParameterModes::from(params.capturemode),
            interval: Fraction::from(params.timeperframe),
        }
    }
}

impl Into<v4l2_captureparm> for CaptureParams {
    fn into(self: CaptureParams) -> v4l2_captureparm {
        let mut params: v4l2_captureparm;
        unsafe {
            params = mem::zeroed();
        }

        params.capability = self.capabilities.into();
        params.capturemode = self.modes.into();
        params.timeperframe = self.interval.into();
        params
    }
}
