use std::fmt;

bitflags::bitflags! {
    #[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
    pub struct Capabilities: u32 {
        const TIME_PER_FRAME    = 0x1000;
    }
}

impl From<u32> for Capabilities {
    fn from(caps: u32) -> Self {
        Self::from_bits_retain(caps)
    }
}

impl From<Capabilities> for u32 {
    fn from(capabilities: Capabilities) -> Self {
        capabilities.bits()
    }
}

impl fmt::Display for Capabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
