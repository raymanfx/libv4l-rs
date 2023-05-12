use std::io;

use crate::video::capture::Parameters as CaptureParameters;
use crate::video::output::Parameters as OutputParameters;
use crate::{
    buffer, format::Description as FormatDescription, format::Format, format::FourCC,
    frameinterval::FrameInterval, framesize::FrameSize,
};

/// Capture device protocol
pub trait Capture {
    /// Returns a vector of all frame intervals that the device supports for the given pixel format
    /// and frame size
    fn enum_frameintervals(
        &self,
        fourcc: FourCC,
        width: u32,
        height: u32,
    ) -> io::Result<Vec<FrameInterval>>;

    /// Returns a vector of valid framesizes that the device supports for the given pixel format
    fn enum_framesizes(&self, fourcc: FourCC) -> io::Result<Vec<FrameSize>>;

    /// Returns a vector of valid formats for this device
    ///
    /// The "emulated" field describes formats filled in by libv4lconvert.
    /// There may be a conversion related performance penalty when using them.
    fn enum_formats(&self) -> io::Result<Vec<FormatDescription>>;

    /// Returns the format currently in use
    fn format(&self) -> io::Result<Format>;

    /// Modifies the capture format and returns the actual format
    ///
    /// The driver tries to match the format parameters on a best effort basis.
    /// Thus, if the combination of format properties cannot be achieved, the closest possible
    /// settings are used and reported back.
    ///
    ///
    /// # Arguments
    ///
    /// * `fmt` - Desired format
    fn set_format(&self, fmt: &Format) -> io::Result<Format>;

    /// Returns the parameters currently in use
    fn params(&self) -> io::Result<CaptureParameters>;

    /// Modifies the capture parameters and returns the actual parameters
    ///
    /// # Arguments
    ///
    /// * `params` - Desired parameters
    fn set_params(&self, params: &CaptureParameters) -> io::Result<CaptureParameters>;
}

/// Output device protocol
pub trait Output {
    /// Returns a vector of all frame intervals that the device supports for the given pixel format
    /// and frame size
    fn enum_frameintervals(
        &self,
        fourcc: FourCC,
        width: u32,
        height: u32,
    ) -> io::Result<Vec<FrameInterval>>;

    /// Returns a vector of valid framesizes that the device supports for the given pixel format
    fn enum_framesizes(&self, fourcc: FourCC) -> io::Result<Vec<FrameSize>>;

    /// Returns a vector of valid formats for this device
    ///
    /// The "emulated" field describes formats filled in by libv4lconvert.
    /// There may be a conversion related performance penalty when using them.
    fn enum_formats(&self) -> io::Result<Vec<FormatDescription>>;

    /// Returns the format currently in use
    fn format(&self) -> io::Result<Format>;

    /// Modifies the capture format and returns the actual format
    ///
    /// The driver tries to match the format parameters on a best effort basis.
    /// Thus, if the combination of format properties cannot be achieved, the closest possible
    /// settings are used and reported back.
    ///
    ///
    /// # Arguments
    ///
    /// * `fmt` - Desired format
    fn set_format(&self, fmt: &Format) -> io::Result<Format>;

    /// Returns the parameters currently in use
    fn params(&self) -> io::Result<OutputParameters>;

    /// Modifies the output parameters and returns the actual parameters
    ///
    /// # Arguments
    ///
    /// * `params` - Desired parameters
    fn set_params(&self, params: &OutputParameters) -> io::Result<OutputParameters>;
}

/// Shared video device protocol
///
/// This trait exists so we can reuse it in the Capture and Output traits. We want to reuse the
/// code for methods such as `enum_formats`, but need to specifiy the buffer type `buf_type` each
/// time. Since we already know the value of `buf_type` at compile time, it makes no sense to
/// place the burden of specifying it on the user.
///
/// Hint: the value is known at compile time because we encode the information in the traits
/// themselves, i.e. `Capture` implies buffer::Type::Capture, etc.
pub(crate) trait Video {
    /// Returns a vector of all frame intervals that the device supports for the given pixel format
    /// and frame size
    fn enum_frameintervals(
        &self,
        fourcc: FourCC,
        width: u32,
        height: u32,
    ) -> io::Result<Vec<FrameInterval>>;

    /// Returns a vector of valid framesizes that the device supports for the given pixel format
    fn enum_framesizes(&self, fourcc: FourCC) -> io::Result<Vec<FrameSize>>;

    /// Returns a vector of valid formats for this device
    ///
    /// The "emulated" field describes formats filled in by libv4lconvert.
    /// There may be a conversion related performance penalty when using them.
    fn enum_formats(&self, typ: buffer::Type) -> io::Result<Vec<FormatDescription>>;

    /// Returns the format currently in use
    fn format(&self, typ: buffer::Type) -> io::Result<Format>;

    /// Modifies the capture format and returns the actual format
    ///
    /// The driver tries to match the format parameters on a best effort basis.
    /// Thus, if the combination of format properties cannot be achieved, the closest possible
    /// settings are used and reported back.
    ///
    ///
    /// # Arguments
    ///
    /// * `fmt` - Desired format
    fn set_format(&self, typ: buffer::Type, fmt: &Format) -> io::Result<Format>;
}
