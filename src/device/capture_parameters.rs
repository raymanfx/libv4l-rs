use bitflags::bitflags;
use std::{fmt, mem};

use crate::v4l_sys::*;
use crate::Fraction;

bitflags! {
    pub struct Capabilities: u32 {
        #[allow(clippy::unreadable_literal)]
        const TIME_PER_FRAME    = 0x1000;
    }
}

impl From<u32> for Capabilities {
    fn from(caps: u32) -> Self {
        Capabilities::from_bits_truncate(caps)
    }
}

impl Into<u32> for Capabilities {
    fn into(self) -> u32 {
        self.bits()
    }
}

impl fmt::Display for Capabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

bitflags! {
    pub struct Modes: u32 {
        const HIGH_QUALITY      = 0x1000;
    }
}

impl From<u32> for Modes {
    fn from(caps: u32) -> Self {
        Modes::from_bits_truncate(caps)
    }
}

impl Into<u32> for Modes {
    fn into(self) -> u32 {
        self.bits()
    }
}

impl fmt::Display for Modes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Copy, Clone)]
/// Streaming parameters (single-planar)
pub struct CaptureParams {
    pub capabilities: Capabilities,
    pub modes: Modes,
    pub interval: Fraction,
}

impl CaptureParams {
    pub fn new(frac: Fraction) -> Self {
        CaptureParams {
            capabilities: Capabilities::from(0),
            modes: Modes::from(0),
            interval: frac,
        }
    }

    pub fn with_fps(fps: u32) -> Self {
        CaptureParams {
            capabilities: Capabilities::from(0),
            modes: Modes::from(0),
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
            capabilities: Capabilities::from(params.capability),
            modes: Modes::from(params.capturemode),
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
