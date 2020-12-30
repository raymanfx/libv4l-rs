use std::convert::TryFrom;
use std::fmt;

use crate::format::FourCC;
use crate::v4l_sys;
use crate::v4l_sys::*;

#[derive(Debug)]
/// Format description as returned by VIDIOC_ENUM_FRAMESIZES
pub struct FrameSize {
    pub index: u32,
    pub fourcc: FourCC,
    pub typ: u32,
    pub size: FrameSizeEnum,
}

impl fmt::Display for FrameSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "index  : {}", self.index)?;
        writeln!(f, "type   : {}", self.typ)?;
        writeln!(f, "fourcc : {}", self.fourcc)?;
        writeln!(f, "size   : {}", self.size)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum FrameSizeEnum {
    Discrete(Discrete),
    Stepwise(Stepwise),
}

impl FrameSizeEnum {
    pub fn to_discrete(self) -> impl IntoIterator<Item = Discrete> {
        match self {
            Self::Discrete(discrete) => vec![discrete],
            Self::Stepwise(stepwise) => {
                let mut discrete = Vec::new();

                for width in
                    (stepwise.min_width..=stepwise.max_width).step_by(stepwise.step_width as usize)
                {
                    for height in (stepwise.min_height..=stepwise.max_height)
                        .step_by(stepwise.step_height as usize)
                    {
                        discrete.push(Discrete {
                            width: width,
                            height: height,
                        });
                    }
                }

                discrete
            }
        }
    }
}

impl fmt::Display for FrameSizeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameSizeEnum::Discrete(val) => write!(f, "Discrete({})", val)?,
            FrameSizeEnum::Stepwise(val) => write!(f, "Stepwise({})", val)?,
        }

        Ok(())
    }
}

impl TryFrom<v4l2_frmsizeenum> for FrameSizeEnum {
    type Error = String;

    fn try_from(desc: v4l2_frmsizeenum) -> Result<Self, Self::Error> {
        unsafe {
            // Unsafe because of access to union __bindgen_anon_1
            match desc.type_ {
                v4l_sys::v4l2_frmsizetypes_V4L2_FRMSIZE_TYPE_DISCRETE => Ok({
                    FrameSizeEnum::Discrete(Discrete {
                        width: desc.__bindgen_anon_1.discrete.width,
                        height: desc.__bindgen_anon_1.discrete.height,
                    })
                }),
                v4l_sys::v4l2_frmsizetypes_V4L2_FRMSIZE_TYPE_STEPWISE
                | v4l_sys::v4l2_frmsizetypes_V4L2_FRMSIZE_TYPE_CONTINUOUS => Ok({
                    FrameSizeEnum::Stepwise(Stepwise {
                        min_width: desc.__bindgen_anon_1.stepwise.min_width,
                        max_width: desc.__bindgen_anon_1.stepwise.max_width,
                        step_width: desc.__bindgen_anon_1.stepwise.step_width,
                        min_height: desc.__bindgen_anon_1.stepwise.min_height,
                        max_height: desc.__bindgen_anon_1.stepwise.max_height,
                        step_height: desc.__bindgen_anon_1.stepwise.step_height,
                    })
                }),
                typ => Err(format!("Unknown frame size type: {}", typ)),
            }
        }
    }
}

#[derive(Debug)]
pub struct Discrete {
    /// Width of the frame [pixel].
    pub width: u32,
    /// Height of the frame [pixel].
    pub height: u32,
}

impl fmt::Display for Discrete {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Stepwise {
    /// Minimum frame width [pixel].
    pub min_width: u32,
    /// Maximum frame width [pixel].
    pub max_width: u32,
    /// Frame width step size [pixel].
    pub step_width: u32,
    /// Minimum frame height [pixel].
    pub min_height: u32,
    /// Maximum frame height [pixel].
    pub max_height: u32,
    /// Frame height step size [pixel].
    pub step_height: u32,
}

impl fmt::Display for Stepwise {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}x{} - {}x{} with step {}/{}",
            self.min_width,
            self.min_height,
            self.max_width,
            self.max_height,
            self.step_width,
            self.step_height,
        )?;
        Ok(())
    }
}

impl TryFrom<v4l2_frmsizeenum> for FrameSize {
    type Error = String;

    fn try_from(desc: v4l2_frmsizeenum) -> Result<Self, Self::Error> {
        Ok(FrameSize {
            index: desc.index,
            typ: desc.type_,
            fourcc: FourCC::from(desc.pixel_format),
            size: FrameSizeEnum::try_from(desc)?,
        })
    }
}
