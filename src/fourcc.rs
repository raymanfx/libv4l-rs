use std::{fmt, str};

#[derive(Debug, Default, Copy, Clone, Eq)]
/// Four character code representing a pixelformat
pub struct FourCC {
    repr: [u8; 4],
}

impl FourCC {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    /// Returns a pixelformat as four character code
    ///
    /// # Arguments
    ///
    /// * `repr` - Four characters as raw bytes
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::FourCC;
    /// let fourcc = FourCC::new(b"YUYV");
    /// ```
    pub fn new(repr: &[u8; 4]) -> FourCC {
        FourCC { repr: *repr }
    }
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = str::from_utf8(&self.repr);
        if let Ok(string) = string {
            write!(f, "{}", string)?;
        }
        Ok(())
    }
}

impl PartialEq for FourCC {
    fn eq(&self, other: &FourCC) -> bool {
        self.repr.iter().zip(other.repr.iter()).all(|(a, b)| a == b)
    }
}

impl From<u32> for FourCC {
    fn from(code: u32) -> Self {
        let mut repr: [u8; 4] = [0; 4];

        repr[0] = (code & 0xff) as u8;
        repr[1] = ((code >> 8) & 0xff) as u8;
        repr[2] = ((code >> 16) & 0xff) as u8;
        repr[3] = ((code >> 24) & 0xff) as u8;
        FourCC::new(&repr)
    }
}

impl Into<u32> for FourCC {
    fn into(self: FourCC) -> u32 {
        let mut code: u32;

        code = self.repr[0] as u32;
        code |= (self.repr[1] as u32) << 8;
        code |= (self.repr[2] as u32) << 16;
        code |= (self.repr[3] as u32) << 24;
        code
    }
}
