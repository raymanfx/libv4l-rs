use bitflags::bitflags;
use std::fmt;

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
