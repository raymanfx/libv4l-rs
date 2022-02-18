use std::{fmt, mem};

use crate::fraction::Fraction;
use crate::parameters::Capabilities;
use crate::v4l_sys::*;

#[derive(Debug, Copy, Clone)]
/// Output parameters (single-planar)
pub struct Parameters {
    pub capabilities: Capabilities,
    pub interval: Fraction,
}

impl Parameters {
    pub fn new(frac: Fraction) -> Self {
        Parameters {
            capabilities: Capabilities::from(0),
            interval: frac,
        }
    }

    pub fn with_fps(fps: u32) -> Self {
        Parameters {
            capabilities: Capabilities::from(0),
            interval: Fraction::new(1, fps),
        }
    }
}

impl fmt::Display for Parameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "capabilities : {}", self.capabilities)?;
        writeln!(f, "interval     : {} [s]", self.interval)?;
        Ok(())
    }
}

impl From<v4l2_outputparm> for Parameters {
    fn from(params: v4l2_outputparm) -> Self {
        Parameters {
            capabilities: Capabilities::from(params.capability),
            interval: Fraction::from(params.timeperframe),
        }
    }
}

impl Into<v4l2_outputparm> for Parameters {
    fn into(self: Parameters) -> v4l2_outputparm {
        v4l2_outputparm {
            capability: self.capabilities.into(),
            timeperframe: self.interval.into(),
            ..unsafe { mem::zeroed() }
        }
    }
}
