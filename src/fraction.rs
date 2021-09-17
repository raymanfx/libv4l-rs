use crate::v4l_sys::*;
use std::fmt;

#[derive(Debug, Default, Clone, Copy)]
/// Fraction used for timing settings
pub struct Fraction {
    pub numerator: u32,
    pub denominator: u32,
}

impl Fraction {
    /// Returns a fraction representation
    ///
    /// # Arguments
    ///
    /// * `num` - Numerator
    /// * `denom` - Denominator
    ///
    /// # Example
    ///
    /// ```
    /// use v4l::fraction::Fraction;
    /// let frac = Fraction::new(30, 1);
    /// ```
    pub fn new(num: u32, denom: u32) -> Self {
        Fraction {
            numerator: num,
            denominator: denom,
        }
    }
}

impl fmt::Display for Fraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
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
        v4l2_fract {
            numerator: self.numerator,
            denominator: self.denominator,
        }
    }
}
