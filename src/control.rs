use std::convert::{TryFrom, TryInto};
use std::{ffi, fmt, mem, str};

use crate::v4l_sys::*;

/// Control data type
#[allow(clippy::unreadable_literal)]
#[rustfmt::skip]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Type {
    Integer         = 1,
    Boolean         = 2,
    Menu            = 3,
    Button          = 4,
    Integer64       = 5,
    CtrlClass       = 6,
    String          = 7,
    Bitmask         = 8,
    IntegerMenu     = 9,

    /* Compound types are >= 0x0100 */
    U8              = 0x0100,
    U16             = 0x0101,
    U32             = 0x0102,
    Area            = 0x0106,

    Unknown(u32),
}

impl From<u32> for Type {
    fn from(repr: u32) -> Self {
        match repr {
            1 => Self::Integer,
            2 => Self::Boolean,
            3 => Self::Menu,
            4 => Self::Button,
            5 => Self::Integer64,
            6 => Self::CtrlClass,
            7 => Self::String,
            8 => Self::Bitmask,
            9 => Self::IntegerMenu,

            0x0100 => Self::U8,
            0x0101 => Self::U16,
            0x0102 => Self::U32,
            0x0106 => Self::Area,
            repr => Self::Unknown(repr),
        }
    }
}

impl From<Type> for u32 {
    fn from(t: Type) -> Self {
        match t {
            Type::Integer => 1,
            Type::Boolean => 2,
            Type::Menu => 3,
            Type::Button => 4,
            Type::Integer64 => 5,
            Type::CtrlClass => 6,
            Type::String => 7,
            Type::Bitmask => 8,
            Type::IntegerMenu => 9,

            Type::U8 => 0x0100,
            Type::U16 => 0x0101,
            Type::U32 => 0x0102,
            Type::Area => 0x0106,
            Type::Unknown(t) => t,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

bitflags::bitflags! {
    #[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
    pub struct Flags: u32 {
        const DISABLED              = 0x0001;
        const GRABBED               = 0x0002;
        const READ_ONLY             = 0x0004;
        const UPDATE                = 0x0008;
        const INACTIVE              = 0x0010;
        const SLIDER                = 0x0020;
        const WRITE_ONLY            = 0x0040;
        const VOLATILE              = 0x0080;
        const HAS_PAYLOAD           = 0x0100;
        const EXECUTE_ON_WRITE      = 0x0200;
        const MODIFY_LAYOUT         = 0x0400;

        const NEXT_CTRL             = 0x80000000;
        const NEXT_COMPOUND         = 0x40000000;
    }
}

impl From<u32> for Flags {
    fn from(flags: u32) -> Self {
        Self::from_bits_retain(flags)
    }
}

impl From<Flags> for u32 {
    fn from(flags: Flags) -> Self {
        flags.bits()
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug)]
/// Device control menu item
pub enum MenuItem {
    Name(String),
    Value(i64),
}

impl fmt::Display for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuItem::Name(name) => {
                write!(f, "{}", name)?;
            }
            MenuItem::Value(value) => {
                write!(f, "{}", value)?;
            }
        }
        Ok(())
    }
}

impl TryFrom<(Type, v4l2_querymenu)> for MenuItem {
    type Error = ();

    fn try_from(item: (Type, v4l2_querymenu)) -> Result<Self, Self::Error> {
        unsafe {
            match item.0 {
                Type::Menu => Ok(MenuItem::Name(
                    str::from_utf8(&item.1.__bindgen_anon_1.name)
                        .unwrap()
                        .trim_matches(char::from(0))
                        .to_string(),
                )),
                Type::IntegerMenu => Ok(MenuItem::Value(item.1.__bindgen_anon_1.value)),
                _ => Err(()),
            }
        }
    }
}

#[derive(Debug)]
/// Device control description
pub struct Description {
    /// Control identifier, set by the the application
    pub id: u32,
    /// Type of control
    pub typ: Type,
    /// Name of the control, intended for the user
    pub name: String,
    /// Minimum value, inclusive
    pub minimum: i64,
    /// Maximum value, inclusive
    pub maximum: i64,
    /// Step size, always positive
    pub step: u64,
    /// Default value
    pub default: i64,
    /// Control flags
    pub flags: Flags,

    /// Items for menu controls (only valid if typ is a menu type)
    pub items: Option<Vec<(u32, MenuItem)>>,
}

impl From<v4l2_query_ext_ctrl> for Description {
    fn from(ctrl: v4l2_query_ext_ctrl) -> Self {
        Self {
            id: ctrl.id,
            typ: Type::from(ctrl.type_),
            name: unsafe { ffi::CStr::from_ptr(ctrl.name.as_ptr()) }
                .to_str()
                .unwrap()
                .to_string(),
            minimum: ctrl.minimum,
            maximum: ctrl.maximum,
            step: ctrl.step,
            default: ctrl.default_value,
            flags: Flags::from(ctrl.flags),
            items: None,
        }
    }
}

impl fmt::Display for Description {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ID         : {}", self.id)?;
        writeln!(f, "Type       : {}", self.typ)?;
        writeln!(f, "Name       : {}", self.name)?;
        writeln!(f, "Minimum    : {}", self.minimum)?;
        writeln!(f, "Maximum    : {}", self.maximum)?;
        writeln!(f, "Step       : {}", self.step)?;
        writeln!(f, "Default    : {}", self.default)?;
        writeln!(f, "Flags      : {}", self.flags)?;
        if let Some(items) = &self.items {
            writeln!(f, "Menu ==>")?;
            for item in items {
                writeln!(f, " * {}", item.1)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Control {
    pub id: u32,
    pub value: Value,
}

#[derive(Debug, PartialEq, Eq)]
/// Device control value
pub enum Value {
    /* buttons */
    None,
    /* single values */
    Integer(i64),
    Boolean(bool),
    String(String),
    /* compound (matrix) values */
    CompoundU8(Vec<u8>),
    CompoundU16(Vec<u16>),
    CompoundU32(Vec<u32>),
    CompoundPtr(Vec<u8>),
}

impl TryInto<v4l2_control> for Control {
    type Error = ();

    fn try_into(self) -> Result<v4l2_control, Self::Error> {
        unsafe {
            let mut ctrl = v4l2_control {
                id: self.id,
                ..mem::zeroed()
            };
            match self.value {
                Value::None => Ok(ctrl),
                Value::Integer(val) => {
                    ctrl.value = val as i32;
                    Ok(ctrl)
                }
                Value::Boolean(val) => {
                    ctrl.value = val as i32;
                    Ok(ctrl)
                }
                _ => Err(()),
            }
        }
    }
}
