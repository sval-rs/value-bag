//! The internal `Value` serialization API.
//!
//! This implementation isn't intended to be public. It may need to change
//! for optimizations or to support new external serialization frameworks.

use crate::{
    ValueBag,
    fill::{Fill, Slot},
    std::any::TypeId,
    Error,
};

pub(super) mod cast;
#[cfg(feature = "error")]
pub(super) mod error;
pub(super) mod fmt;
#[cfg(feature = "serde1")]
pub(super) mod serde;
#[cfg(feature = "sval1")]
pub(super) mod sval;

// NOTE: It takes less space to have separate variants for the presence
// of a `TypeId` instead of using `Option<T>`, because `TypeId` doesn't
// have a niche value
/// A container for a structured value for a specific kind of visitor.
#[derive(Clone, Copy)]
pub(super) enum Internal<'v> {
    /// A simple primitive value that can be copied without allocating.
    Primitive { value: Primitive<'v> },
    /// A value that can be filled.
    Fill { value: &'v dyn Fill },

    /// A debuggable value.
    AnonDebug {
        value: &'v dyn fmt::Debug,
    },
    /// A debuggable value.
    Debug {
        value: &'v dyn fmt::Debug,
        type_id: TypeId,
    },

    /// A displayable value.
    AnonDisplay {
        value: &'v dyn fmt::Display,
    },
    /// A displayable value.
    Display {
        value: &'v dyn fmt::Display,
        type_id: TypeId,
    },

    #[cfg(feature = "error")]
    /// An error.
    AnonError {
        value: &'v (dyn error::Error + 'static),
    },
    #[cfg(feature = "error")]
    /// An error.
    Error {
        value: &'v (dyn error::Error + 'static),
        type_id: TypeId,
    },

    #[cfg(feature = "sval1")]
    /// A structured value from `sval`.
    AnonSval1 {
        value: &'v dyn sval::v1::Value,
    },
    #[cfg(feature = "sval1")]
    /// A structured value from `sval`.
    Sval1 {
        value: &'v dyn sval::v1::Value,
        type_id: TypeId,
    },

    #[cfg(feature = "serde1")]
    /// A structured value from `serde`.
    AnonSerde1 {
        value: &'v dyn serde::v1::Serialize,
    },
    #[cfg(feature = "serde1")]
    /// A structured value from `serde`.
    Serde1 {
        value: &'v dyn serde::v1::Serialize,
        type_id: TypeId,
    },
}

/// A captured primitive value.
///
/// These values are common and cheap to copy around.
#[derive(Clone, Copy)]
pub(super) enum Primitive<'v> {
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(&'v str),
    None,
}

impl<'v> ValueBag<'v> {
    /// Get a value from an internal primitive.
    pub(super) fn from_primitive<T>(value: T) -> Self
        where
            T: Into<Primitive<'v>>,
    {
        ValueBag {
            inner: Internal::Primitive {
                value: value.into(),
            },
        }
    }

    /// Visit the value using an internal visitor.
    pub(super) fn internal_visit(&self, visitor: &mut dyn InternalVisitor<'v>) -> Result<(), Error> {
        self.inner.internal_visit(visitor)
    }
}

impl<'v> Internal<'v> {
    pub(super) fn internal_visit(self, visitor: &mut dyn InternalVisitor<'v>) -> Result<(), Error> {
        match self {
            Internal::Primitive { value } => value.internal_visit(visitor),

            Internal::Fill { value } => value.fill(&mut Slot::new(visitor)),

            Internal::AnonDebug { value } => visitor.debug(value),
            Internal::Debug { value, .. } => visitor.debug(value),

            Internal::AnonDisplay { value } => visitor.display(value),
            Internal::Display { value, .. } => visitor.display(value),

            #[cfg(feature = "error")]
            Internal::AnonError { value } => visitor.borrowed_error(value),
            #[cfg(feature = "error")]
            Internal::Error { value, .. } => visitor.borrowed_error(value),

            #[cfg(feature = "sval1")]
            Internal::AnonSval1 { value } => visitor.sval1(value),
            #[cfg(feature = "sval1")]
            Internal::Sval1 { value, .. } => visitor.sval1(value),

            #[cfg(feature = "serde1")]
            Internal::AnonSerde1 { value } => visitor.serde1(value),
            #[cfg(feature = "serde1")]
            Internal::Serde1 { value, .. } => visitor.serde1(value),
        }
    }
}

/// The internal serialization contract.
pub(super) trait InternalVisitor<'v> {
    fn debug(&mut self, v: &dyn fmt::Debug) -> Result<(), Error>;
    fn display(&mut self, v: &dyn fmt::Display) -> Result<(), Error> {
        self.debug(&format_args!("{}", v))
    }

    fn u64(&mut self, v: u64) -> Result<(), Error>;
    fn i64(&mut self, v: i64) -> Result<(), Error>;
    fn f64(&mut self, v: f64) -> Result<(), Error>;
    fn bool(&mut self, v: bool) -> Result<(), Error>;
    fn char(&mut self, v: char) -> Result<(), Error>;

    fn str(&mut self, v: &str) -> Result<(), Error>;
    fn borrowed_str(&mut self, v: &'v str) -> Result<(), Error> {
        self.str(v)
    }

    fn none(&mut self) -> Result<(), Error>;

    #[cfg(feature = "error")]
    fn error(&mut self, v: &(dyn error::Error + 'static)) -> Result<(), Error>;
    #[cfg(feature = "error")]
    fn borrowed_error(&mut self, v: &'v (dyn error::Error + 'static)) -> Result<(), Error> {
        self.error(v)
    }

    #[cfg(feature = "sval1")]
    fn sval1(&mut self, v: &dyn sval::v1::Value) -> Result<(), Error>;

    #[cfg(feature = "serde1")]
    fn serde1(&mut self, v: &dyn serde::v1::Serialize) -> Result<(), Error>;
}

impl<'v> Primitive<'v> {
    fn internal_visit(self, visitor: &mut dyn InternalVisitor<'v>) -> Result<(), Error> {
        match self {
            Primitive::Signed(value) => visitor.i64(value),
            Primitive::Unsigned(value) => visitor.u64(value),
            Primitive::Float(value) => visitor.f64(value),
            Primitive::Bool(value) => visitor.bool(value),
            Primitive::Char(value) => visitor.char(value),
            Primitive::Str(value) => visitor.borrowed_str(value),
            Primitive::None => visitor.none(),
        }
    }
}

impl<'v> From<()> for Primitive<'v> {
    #[inline]
    fn from(_: ()) -> Self {
        Primitive::None
    }
}

impl<'v> From<u8> for Primitive<'v> {
    #[inline]
    fn from(v: u8) -> Self {
        Primitive::Unsigned(v as u64)
    }
}

impl<'v> From<u16> for Primitive<'v> {
    #[inline]
    fn from(v: u16) -> Self {
        Primitive::Unsigned(v as u64)
    }
}

impl<'v> From<u32> for Primitive<'v> {
    #[inline]
    fn from(v: u32) -> Self {
        Primitive::Unsigned(v as u64)
    }
}

impl<'v> From<u64> for Primitive<'v> {
    #[inline]
    fn from(v: u64) -> Self {
        Primitive::Unsigned(v)
    }
}

impl<'v> From<usize> for Primitive<'v> {
    #[inline]
    fn from(v: usize) -> Self {
        Primitive::Unsigned(v as u64)
    }
}

impl<'v> From<i8> for Primitive<'v> {
    #[inline]
    fn from(v: i8) -> Self {
        Primitive::Signed(v as i64)
    }
}

impl<'v> From<i16> for Primitive<'v> {
    #[inline]
    fn from(v: i16) -> Self {
        Primitive::Signed(v as i64)
    }
}

impl<'v> From<i32> for Primitive<'v> {
    #[inline]
    fn from(v: i32) -> Self {
        Primitive::Signed(v as i64)
    }
}

impl<'v> From<i64> for Primitive<'v> {
    #[inline]
    fn from(v: i64) -> Self {
        Primitive::Signed(v)
    }
}

impl<'v> From<isize> for Primitive<'v> {
    #[inline]
    fn from(v: isize) -> Self {
        Primitive::Signed(v as i64)
    }
}

impl<'v> From<f32> for Primitive<'v> {
    #[inline]
    fn from(v: f32) -> Self {
        Primitive::Float(v as f64)
    }
}

impl<'v> From<f64> for Primitive<'v> {
    #[inline]
    fn from(v: f64) -> Self {
        Primitive::Float(v)
    }
}

impl<'v> From<bool> for Primitive<'v> {
    #[inline]
    fn from(v: bool) -> Self {
        Primitive::Bool(v)
    }
}

impl<'v> From<char> for Primitive<'v> {
    #[inline]
    fn from(v: char) -> Self {
        Primitive::Char(v)
    }
}

impl<'v> From<&'v str> for Primitive<'v> {
    #[inline]
    fn from(v: &'v str) -> Self {
        Primitive::Str(v)
    }
}
