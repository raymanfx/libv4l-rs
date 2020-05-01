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
    /// use v4l::Fraction;
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
