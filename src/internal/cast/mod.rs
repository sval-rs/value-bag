//! Coerce a `Value` into some concrete types.
//!
//! These operations are cheap when the captured value is a simple primitive,
//! but may end up executing arbitrary caller code if the value is complex.
//! They will also attempt to downcast erased types into a primitive where possible.

use crate::std::{
    any::TypeId,
    convert::{TryFrom, TryInto},
    fmt,
};

#[cfg(feature = "std")]
use crate::std::{borrow::ToOwned, string::String};

use super::{Internal, InternalVisitor, Primitive};
use crate::{Error, ValueBag};

mod primitive;

pub(super) fn type_id<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}

/// Attempt to capture a primitive from some generic value.
///
/// If the value is a primitive type, then cast it here, avoiding needing to erase its value
/// This makes `ValueBag`s produced by `ValueBag::from_*` more useful
pub(super) fn try_from_primitive<'v, T: 'static>(value: &'v T) -> Option<ValueBag<'v>> {
    primitive::from_any(value).map(|primitive| ValueBag {
        inner: Internal::Primitive { value: primitive },
    })
}

impl<'v> ValueBag<'v> {
    /// Try get a `u64` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_u64(&self) -> Option<u64> {
        self.inner.cast().into_primitive().into_u64()
    }

    /// Try get a `i64` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_i64(&self) -> Option<i64> {
        self.inner.cast().into_primitive().into_i64()
    }

    /// Try get a `u128` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_u128(&self) -> Option<u128> {
        self.inner.cast().into_primitive().into_u128()
    }

    /// Try get a `i128` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_i128(&self) -> Option<i128> {
        self.inner.cast().into_primitive().into_i128()
    }

    /// Try get a `f64` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_f64(&self) -> Option<f64> {
        self.inner.cast().into_primitive().into_f64()
    }

    /// Try get a `bool` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_bool(&self) -> Option<bool> {
        self.inner.cast().into_primitive().into_bool()
    }

    /// Try get a `char` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_char(&self) -> Option<char> {
        self.inner.cast().into_primitive().into_char()
    }

    /// Try get a `str` from this value.
    ///
    /// This method is cheap for primitive types. It won't allocate an owned
    /// `String` if the value is a complex type.
    pub fn to_borrowed_str(&self) -> Option<&str> {
        self.inner.cast().into_primitive().into_borrowed_str()
    }

    /// Check whether this value can be downcast to `T`.
    pub fn is<T: 'static>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Try downcast this value to `T`.
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        let target = TypeId::of::<T>();
        match self.inner {
            Internal::Debug { type_id, value } if type_id == target => {
                Some(unsafe { &*(value as *const _ as *const T) })
            }
            Internal::Display { type_id, value } if type_id == target => {
                Some(unsafe { &*(value as *const _ as *const T) })
            }
            #[cfg(feature = "error")]
            Internal::Error { type_id, value } if type_id == target => {
                Some(unsafe { &*(value as *const _ as *const T) })
            }
            #[cfg(feature = "sval1")]
            Internal::Sval1 { type_id, value } if type_id == target => {
                Some(unsafe { &*(value as *const _ as *const T) })
            }
            #[cfg(feature = "serde1")]
            Internal::Serde1 { type_id, value } if type_id == target => {
                Some(unsafe { &*(value as *const _ as *const T) })
            }
            _ => None,
        }
    }
}

impl<'v> Internal<'v> {
    /// Cast the inner value to another type.
    fn cast(self) -> Cast<'v> {
        struct CastVisitor<'v>(Cast<'v>);

        impl<'v> InternalVisitor<'v> for CastVisitor<'v> {
            fn debug(&mut self, _: &dyn fmt::Debug) -> Result<(), Error> {
                Ok(())
            }

            fn display(&mut self, _: &dyn fmt::Display) -> Result<(), Error> {
                Ok(())
            }

            fn u64(&mut self, v: u64) -> Result<(), Error> {
                self.0 = Cast::Primitive(Primitive::Unsigned(v));
                Ok(())
            }

            fn i64(&mut self, v: i64) -> Result<(), Error> {
                self.0 = Cast::Primitive(Primitive::Signed(v));
                Ok(())
            }

            fn f64(&mut self, v: f64) -> Result<(), Error> {
                self.0 = Cast::Primitive(Primitive::Float(v));
                Ok(())
            }

            fn i128(&mut self, v: i128) -> Result<(), Error> {
                self.0 = Cast::Primitive(Primitive::BigSigned(v));
                Ok(())
            }

            fn u128(&mut self, v: u128) -> Result<(), Error> {
                self.0 = Cast::Primitive(Primitive::BigUnsigned(v));
                Ok(())
            }

            fn bool(&mut self, v: bool) -> Result<(), Error> {
                self.0 = Cast::Primitive(Primitive::Bool(v));
                Ok(())
            }

            fn char(&mut self, v: char) -> Result<(), Error> {
                self.0 = Cast::Primitive(Primitive::Char(v));
                Ok(())
            }

            #[cfg(feature = "std")]
            fn str(&mut self, s: &str) -> Result<(), Error> {
                self.0 = Cast::String(s.to_owned());
                Ok(())
            }

            #[cfg(not(feature = "std"))]
            fn str(&mut self, _: &str) -> Result<(), Error> {
                Ok(())
            }

            fn borrowed_str(&mut self, v: &'v str) -> Result<(), Error> {
                self.0 = Cast::Primitive(Primitive::Str(v));
                Ok(())
            }

            fn none(&mut self) -> Result<(), Error> {
                self.0 = Cast::Primitive(Primitive::None);
                Ok(())
            }

            #[cfg(feature = "error")]
            fn error(&mut self, _: &dyn super::error::Error) -> Result<(), Error> {
                Ok(())
            }

            #[cfg(feature = "sval1")]
            fn sval1(&mut self, v: &dyn super::sval::v1::Value) -> Result<(), Error> {
                super::sval::v1::internal_visit(v, self)
            }

            #[cfg(feature = "serde1")]
            fn serde1(&mut self, v: &dyn super::serde::v1::Serialize) -> Result<(), Error> {
                super::serde::v1::internal_visit(v, self)
            }
        }

        if let Internal::Primitive { value } = self {
            Cast::Primitive(value)
        } else {
            // If the erased value isn't a primitive then we visit it
            let mut cast = CastVisitor(Cast::Primitive(Primitive::None));
            let _ = self.internal_visit(&mut cast);
            cast.0
        }
    }
}

pub(in crate::internal) enum Cast<'v> {
    Primitive(Primitive<'v>),
    #[cfg(feature = "std")]
    String(String),
}

impl<'v> Cast<'v> {
    fn into_primitive(self) -> Primitive<'v> {
        match self {
            Cast::Primitive(value) => value,
            #[cfg(feature = "std")]
            _ => Primitive::None,
        }
    }
}

impl<'v> Primitive<'v> {
    fn into_borrowed_str(self) -> Option<&'v str> {
        if let Primitive::Str(value) = self {
            Some(value)
        } else {
            None
        }
    }

    fn into_u64(self) -> Option<u64> {
        match self {
            Primitive::Unsigned(value) => Some(value),
            Primitive::BigUnsigned(value) => value.try_into().ok(),
            Primitive::Signed(value) => value.try_into().ok(),
            Primitive::BigSigned(value) => value.try_into().ok(),
            _ => None,
        }
    }

    fn into_i64(self) -> Option<i64> {
        match self {
            Primitive::Signed(value) => Some(value),
            Primitive::BigSigned(value) => value.try_into().ok(),
            Primitive::Unsigned(value) => value.try_into().ok(),
            Primitive::BigUnsigned(value) => value.try_into().ok(),
            _ => None,
        }
    }

    fn into_u128(self) -> Option<u128> {
        match self {
            Primitive::BigUnsigned(value) => Some(value),
            Primitive::Unsigned(value) => Some(value.into()),
            Primitive::Signed(value) => value.try_into().ok(),
            Primitive::BigSigned(value) => value.try_into().ok(),
            _ => None,
        }
    }

    fn into_i128(self) -> Option<i128> {
        match self {
            Primitive::BigSigned(value) => Some(value),
            Primitive::Signed(value) => Some(value.into()),
            Primitive::Unsigned(value) => value.try_into().ok(),
            Primitive::BigUnsigned(value) => value.try_into().ok(),
            _ => None,
        }
    }

    fn into_f64(self) -> Option<f64> {
        match self {
            Primitive::Float(value) => Some(value),
            Primitive::Unsigned(value) => u32::try_from(value)
                .ok()
                .and_then(|value| value.try_into().ok()),
            Primitive::Signed(value) => i32::try_from(value)
                .ok()
                .and_then(|value| value.try_into().ok()),
            Primitive::BigUnsigned(value) => u32::try_from(value)
                .ok()
                .and_then(|value| value.try_into().ok()),
            Primitive::BigSigned(value) => i32::try_from(value)
                .ok()
                .and_then(|value| value.try_into().ok()),
            _ => None,
        }
    }

    fn into_char(self) -> Option<char> {
        if let Primitive::Char(value) = self {
            Some(value)
        } else {
            None
        }
    }

    fn into_bool(self) -> Option<bool> {
        if let Primitive::Bool(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use crate::std::borrow::Cow;

    impl<'v> ValueBag<'v> {
        /// Try get a `str` from this value.
        ///
        /// This method is cheap for primitive types, but may call arbitrary
        /// serialization implementations for complex ones. If the serialization
        /// implementation produces a short lived string it will be allocated.
        pub fn to_str(&self) -> Option<Cow<str>> {
            self.inner.cast().into_str()
        }
    }

    impl<'v> Cast<'v> {
        pub(super) fn into_str(self) -> Option<Cow<'v, str>> {
            match self {
                Cast::Primitive(Primitive::Str(value)) => Some(value.into()),
                Cast::String(value) => Some(value.into()),
                _ => None,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        #[cfg(target_arch = "wasm32")]
        use wasm_bindgen_test::*;

        use crate::{std::borrow::ToOwned, test::IntoValueBag};

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn primitive_cast() {
            let short_lived = "a string".to_owned();
            assert_eq!(
                "a string",
                (&*short_lived)
                    .into_value_bag()
                    .to_borrowed_str()
                    .expect("invalid value")
            );
            assert_eq!(
                "a string",
                &*"a string".into_value_bag().to_str().expect("invalid value")
            );
            assert_eq!(
                "a string",
                (&*short_lived)
                    .into_value_bag()
                    .to_str()
                    .expect("invalid value")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    use crate::test::IntoValueBag;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn primitive_cast() {
        assert_eq!(
            "a string",
            "a string"
                .into_value_bag()
                .to_borrowed_str()
                .expect("invalid value")
        );

        assert_eq!(1u64, 1u8.into_value_bag().to_u64().expect("invalid value"));
        assert_eq!(1u64, 1u16.into_value_bag().to_u64().expect("invalid value"));
        assert_eq!(1u64, 1u32.into_value_bag().to_u64().expect("invalid value"));
        assert_eq!(1u64, 1u64.into_value_bag().to_u64().expect("invalid value"));
        assert_eq!(
            1u64,
            1usize.into_value_bag().to_u64().expect("invalid value")
        );
        assert_eq!(
            1u128,
            1u128.into_value_bag().to_u128().expect("invalid value")
        );

        assert_eq!(
            -1i64,
            -1i8.into_value_bag().to_i64().expect("invalid value")
        );
        assert_eq!(
            -1i64,
            -1i8.into_value_bag().to_i64().expect("invalid value")
        );
        assert_eq!(
            -1i64,
            -1i8.into_value_bag().to_i64().expect("invalid value")
        );
        assert_eq!(
            -1i64,
            -1i64.into_value_bag().to_i64().expect("invalid value")
        );
        assert_eq!(
            -1i64,
            -1isize.into_value_bag().to_i64().expect("invalid value")
        );
        assert_eq!(
            -1i128,
            -1i128.into_value_bag().to_i128().expect("invalid value")
        );

        assert!(1f64.into_value_bag().to_f64().is_some());
        assert!(1u64.into_value_bag().to_f64().is_some());
        assert!((-1i64).into_value_bag().to_f64().is_some());
        assert!(1u128.into_value_bag().to_f64().is_some());
        assert!((-1i128).into_value_bag().to_f64().is_some());

        assert!(u64::MAX.into_value_bag().to_u128().is_some());
        assert!(i64::MIN.into_value_bag().to_i128().is_some());
        assert!(i64::MAX.into_value_bag().to_u64().is_some());

        assert!((-1i64).into_value_bag().to_u64().is_none());
        assert!(u64::MAX.into_value_bag().to_i64().is_none());
        assert!(u64::MAX.into_value_bag().to_f64().is_none());

        assert!(i128::MAX.into_value_bag().to_i64().is_none());
        assert!(u128::MAX.into_value_bag().to_u64().is_none());

        assert!(1f64.into_value_bag().to_u64().is_none());

        assert_eq!('a', 'a'.into_value_bag().to_char().expect("invalid value"));
        assert_eq!(
            true,
            true.into_value_bag().to_bool().expect("invalid value")
        );
    }
}
