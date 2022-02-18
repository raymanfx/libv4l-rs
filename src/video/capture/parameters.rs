use bitflags::bitflags;
use std::{fmt, mem};

use crate::fraction::Fraction;
use crate::parameters::Capabilities;
use crate::v4l_sys::*;

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
pub struct Parameters {
    pub capabilities: Capabilities,
    pub modes: Modes,
    pub interval: Fraction,
}

impl Parameters {
    pub fn new(frac: Fraction) -> Self {
        Parameters {
            capabilities: Capabilities::from(0),
            modes: Modes::from(0),
            interval: frac,
        }
    }

    pub fn with_fps(fps: u32) -> Self {
        Parameters {
            capabilities: Capabilities::from(0),
            modes: Modes::from(0),
            interval: Fraction::new(1, fps),
        }
    }
}

impl fmt::Display for Parameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "capabilities : {}", self.capabilities)?;
        writeln!(f, "modes        : {}", self.modes)?;
        writeln!(f, "interval     : {} [s]", self.interval)?;
        Ok(())
    }
}

impl From<v4l2_captureparm> for Parameters {
    fn from(params: v4l2_captureparm) -> Self {
        Parameters {
            capabilities: Capabilities::from(params.capability),
            modes: Modes::from(params.capturemode),
            interval: Fraction::from(params.timeperframe),
        }
    }
}

impl Into<v4l2_captureparm> for Parameters {
    fn into(self: Parameters) -> v4l2_captureparm {
        v4l2_captureparm {
            capability: self.capabilities.into(),
            capturemode: self.modes.into(),
            timeperframe: self.interval.into(),
            ..unsafe { mem::zeroed() }
        }
    }
}
