//! Coerce a `Value` into some concrete types.
//!
//! These operations are cheap when the captured value is a simple primitive,
//! but may end up executing arbitrary caller code if the value is complex.
//! They will also attempt to downcast erased types into a primitive where possible.

use core::marker::PhantomData;

use crate::std::{
    convert::{TryFrom, TryInto},
    fmt,
};

#[cfg(feature = "alloc")]
use crate::std::string::String;

use super::{Internal, InternalVisitor};
use crate::{Error, ValueBag};

mod primitive;

impl<'v> ValueBag<'v> {
    /// Try capture a raw value.
    ///
    /// This method will return `Some` if the value is a simple primitive
    /// that can be captured without losing its structure. In other cases
    /// this method will return `None`.
    pub fn try_capture<T>(value: &'v T) -> Option<Self>
    where
        T: ?Sized + 'static,
    {
        primitive::from_any(value)
    }

    /// Try get a `u64` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_u64(&self) -> Option<u64> {
        self.inner.cast().into_u64()
    }

    /// Try push nested values as `u64`s from this value into the given collection.
    ///
    /// If this value is a primitive type then this method is equivalent to `to_u64`.
    /// If this value is a sequence then each element will be cast to a `u64`.
    /// Any elements that fail to cast will be passed as `None`s.
    pub fn collect_u64(&self, into: &mut (impl Extend<Option<u64>> + ?Sized)) {
        self.inner.collect(into, |cast| cast.into_u64())
    }

    /// Try get a `i64` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_i64(&self) -> Option<i64> {
        self.inner.cast().into_i64()
    }

    /// Try push nested values as `i64`s from this value into the given collection.
    ///
    /// If this value is a primitive type then this method is equivalent to `to_i64`.
    /// If this value is a sequence then each element will be cast to a `i64`.
    /// Any elements that fail to cast will be passed as `None`s.
    pub fn collect_i64(&self, into: &mut (impl Extend<Option<i64>> + ?Sized)) {
        self.inner.collect(into, |cast| cast.into_i64())
    }

    /// Try get a `u128` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_u128(&self) -> Option<u128> {
        self.inner.cast().into_u128()
    }

    /// Try push nested values as `u128`s from this value into the given collection.
    ///
    /// If this value is a primitive type then this method is equivalent to `to_u128`.
    /// If this value is a sequence then each element will be cast to a `u128`.
    /// Any elements that fail to cast will be passed as `None`s.
    pub fn collect_u128(&self, into: &mut (impl Extend<Option<u128>> + ?Sized)) {
        self.inner.collect(into, |cast| cast.into_u128())
    }

    /// Try get a `i128` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_i128(&self) -> Option<i128> {
        self.inner.cast().into_i128()
    }

    /// Try push nested values as `i128`s from this value into the given collection.
    ///
    /// If this value is a primitive type then this method is equivalent to `to_i128`.
    /// If this value is a sequence then each element will be cast to a `i128`.
    /// Any elements that fail to cast will be passed as `None`s.
    pub fn collect_i128(&self, into: &mut (impl Extend<Option<i128>> + ?Sized)) {
        self.inner.collect(into, |cast| cast.into_i128())
    }

    /// Try get a `f64` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_f64(&self) -> Option<f64> {
        self.inner.cast().into_f64()
    }

    /// Try push nested values as `f64`s from this value into the given collection.
    ///
    /// If this value is a primitive type then this method is equivalent to `to_f64`.
    /// If this value is a sequence then each element will be cast to a `f64`.
    /// Any elements that fail to cast will be passed as `None`s.
    pub fn collect_f64(&self, into: &mut (impl Extend<Option<f64>> + ?Sized)) {
        self.inner.collect(into, |cast| cast.into_f64())
    }

    /// Try get a `bool` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_bool(&self) -> Option<bool> {
        self.inner.cast().into_bool()
    }

    /// Try push nested values as `bool`s from this value into the given collection.
    ///
    /// If this value is a primitive type then this method is equivalent to `to_bool`.
    /// If this value is a sequence then each element will be cast to a `bool`.
    /// Any elements that fail to cast will be passed as `None`s.
    pub fn collect_bool(&self, into: &mut (impl Extend<Option<bool>> + ?Sized)) {
        self.inner.collect(into, |cast| cast.into_bool())
    }

    /// Try get a `char` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_char(&self) -> Option<char> {
        self.inner.cast().into_char()
    }

    /// Try push nested values as `char`s from this value into the given collection.
    ///
    /// If this value is a primitive type then this method is equivalent to `to_char`.
    /// If this value is a sequence then each element will be cast to a `char`.
    /// Any elements that fail to cast will be passed as `None`s.
    pub fn collect_char(&self, into: &mut (impl Extend<Option<char>> + ?Sized)) {
        self.inner.collect(into, |cast| cast.into_char())
    }

    /// Try get a `str` from this value.
    ///
    /// This method is cheap for primitive types. It won't allocate an owned
    /// `String` if the value is a complex type.
    pub fn to_borrowed_str(&self) -> Option<&'v str> {
        self.inner.cast().into_borrowed_str()
    }

    /// Try push nested values as `str`s from this value into the given collection.
    ///
    /// If this value is a primitive type then this method is equivalent to `to_borrowed_str`.
    /// If this value is a sequence then each element will be cast to a `str`.
    /// Any elements that fail to cast will be passed as `None`s.
    pub fn collect_borrowed_str(&self, into: &mut (impl Extend<Option<&'v str>> + ?Sized)) {
        self.inner.collect(into, |cast| cast.into_borrowed_str())
    }

    /// Check whether this value can be downcast to `T`.
    pub fn is<T: 'static>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Try downcast this value to `T`.
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        match self.inner {
            Internal::Debug(value) => value.as_any().downcast_ref(),
            Internal::Display(value) => value.as_any().downcast_ref(),
            #[cfg(feature = "error")]
            Internal::Error(value) => value.as_any().downcast_ref(),
            #[cfg(feature = "sval2")]
            Internal::Sval2(value) => value.as_any().downcast_ref(),
            #[cfg(feature = "serde1")]
            Internal::Serde1(value) => value.as_any().downcast_ref(),
            _ => None,
        }
    }
}

impl<'v> Internal<'v> {
    /// Cast the inner value to another type.
    #[inline]
    fn cast(&self) -> Cast<'v> {
        struct CastVisitor<'v> {
            cast: Cast<'v>,
        }

        impl<'v> CastVisitor<'v> {
            fn set(&mut self, cast: Cast<'v>) -> Result<(), Error> {
                self.cast = cast;
                Ok(())
            }
        }

        impl<'v> InternalVisitor<'v> for CastVisitor<'v> {
            #[inline]
            fn debug(&mut self, _: &dyn fmt::Debug) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn display(&mut self, _: &dyn fmt::Display) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn seq_elem(&mut self, _: ValueBag) -> Result<(), Error> {
                self.cast = Cast::None;
                Err(Error::msg("cannot cast complex values"))
            }

            #[inline]
            fn u64(&mut self, v: u64) -> Result<(), Error> {
                self.set(Cast::Unsigned(v))
            }

            #[inline]
            fn i64(&mut self, v: i64) -> Result<(), Error> {
                self.set(Cast::Signed(v))
            }

            #[inline]
            fn u128(&mut self, v: &u128) -> Result<(), Error> {
                self.set(Cast::BigUnsigned(*v))
            }

            #[inline]
            fn i128(&mut self, v: &i128) -> Result<(), Error> {
                self.set(Cast::BigSigned(*v))
            }

            #[inline]
            fn f64(&mut self, v: f64) -> Result<(), Error> {
                self.set(Cast::Float(v))
            }

            #[inline]
            fn bool(&mut self, v: bool) -> Result<(), Error> {
                self.set(Cast::Bool(v))
            }

            #[inline]
            fn char(&mut self, v: char) -> Result<(), Error> {
                self.set(Cast::Char(v))
            }

            #[inline]
            fn str(&mut self, s: &str) -> Result<(), Error> {
                self.set(Cast::Str(s).into_owned().unwrap_or(Cast::None))
            }

            #[inline]
            fn borrowed_str(&mut self, v: &'v str) -> Result<(), Error> {
                self.set(Cast::Str(v))
            }

            #[inline]
            fn none(&mut self) -> Result<(), Error> {
                self.set(Cast::None)
            }

            #[cfg(feature = "error")]
            #[inline]
            fn error(&mut self, _: &dyn super::error::Error) -> Result<(), Error> {
                Ok(())
            }

            #[cfg(feature = "sval2")]
            #[inline]
            fn sval2(&mut self, v: &dyn super::sval::v2::Value) -> Result<(), Error> {
                super::sval::v2::internal_visit(v, self)
            }

            #[cfg(feature = "sval2")]
            fn borrowed_sval2(&mut self, v: &'v dyn super::sval::v2::Value) -> Result<(), Error> {
                super::sval::v2::borrowed_internal_visit(v, self)
            }

            #[cfg(feature = "serde1")]
            #[inline]
            fn serde1(&mut self, v: &dyn super::serde::v1::Serialize) -> Result<(), Error> {
                super::serde::v1::internal_visit(v, self)
            }

            fn poisoned(&mut self, _: &'static str) -> Result<(), Error> {
                self.cast = Cast::None;
                Ok(())
            }
        }

        match &self {
            Internal::Signed(value) => Cast::Signed(*value),
            Internal::Unsigned(value) => Cast::Unsigned(*value),
            #[cfg(feature = "inline-i128")]
            Internal::BigSigned(value) => Cast::BigSigned(*value),
            #[cfg(not(feature = "inline-i128"))]
            Internal::BigSigned(value) => Cast::BigSigned(**value),
            #[cfg(feature = "inline-i128")]
            Internal::BigUnsigned(value) => Cast::BigUnsigned(*value),
            #[cfg(not(feature = "inline-i128"))]
            Internal::BigUnsigned(value) => Cast::BigUnsigned(**value),
            Internal::Float(value) => Cast::Float(*value),
            Internal::Bool(value) => Cast::Bool(*value),
            Internal::Char(value) => Cast::Char(*value),
            Internal::Str(value) => Cast::Str(*value),
            Internal::None => Cast::None,
            other => {
                // If the erased value isn't a primitive then we visit it
                let mut visitor = CastVisitor { cast: Cast::None };
                let _ = other.internal_visit(&mut visitor);
                visitor.cast
            }
        }
    }

    fn collect<T, F: Fn(Cast<'v>) -> Option<T>, C: Extend<Option<T>> + ?Sized>(
        &self,
        collection: &mut C,
        cast: F,
    ) {
        struct Visitor<'a, T, F, C: ?Sized>(&'a mut C, F, PhantomData<T>);

        impl<'a, 'v, T, F, C> InternalVisitor<'v> for Visitor<'a, T, F, C>
        where
            F: Fn(Cast<'v>) -> Option<T>,
            C: Extend<Option<T>> + ?Sized,
        {
            fn debug(&mut self, _: &dyn fmt::Debug) -> Result<(), Error> {
                self.0.extend(Some((self.1)(Cast::None)));

                Ok(())
            }

            fn display(&mut self, _: &dyn fmt::Display) -> Result<(), Error> {
                self.0.extend(Some((self.1)(Cast::None)));

                Ok(())
            }

            fn seq_elem(&mut self, v: ValueBag) -> Result<(), Error> {
                self.0.extend(Some((self.1)(
                    v.inner.cast().into_owned().unwrap_or(Cast::None),
                )));

                Ok(())
            }

            fn borrowed_seq_elem(&mut self, v: ValueBag<'v>) -> Result<(), Error> {
                self.0.extend(Some((self.1)(v.inner.cast())));

                Ok(())
            }

            fn u64(&mut self, v: u64) -> Result<(), Error> {
                self.0.extend(Some((self.1)(Cast::Unsigned(v))));

                Ok(())
            }

            fn i64(&mut self, v: i64) -> Result<(), Error> {
                self.0.extend(Some((self.1)(Cast::Signed(v))));

                Ok(())
            }

            fn u128(&mut self, v: &u128) -> Result<(), Error> {
                self.0.extend(Some((self.1)(Cast::BigUnsigned(*v))));

                Ok(())
            }

            fn i128(&mut self, v: &i128) -> Result<(), Error> {
                self.0.extend(Some((self.1)(Cast::BigSigned(*v))));

                Ok(())
            }

            fn f64(&mut self, v: f64) -> Result<(), Error> {
                self.0.extend(Some((self.1)(Cast::Float(v))));

                Ok(())
            }

            fn bool(&mut self, v: bool) -> Result<(), Error> {
                self.0.extend(Some((self.1)(Cast::Bool(v))));

                Ok(())
            }

            fn char(&mut self, v: char) -> Result<(), Error> {
                self.0.extend(Some((self.1)(Cast::Char(v))));

                Ok(())
            }

            fn str(&mut self, v: &str) -> Result<(), Error> {
                self.0.extend(Some((self.1)(
                    Cast::Str(v).into_owned().unwrap_or(Cast::None),
                )));

                Ok(())
            }

            fn borrowed_str(&mut self, v: &'v str) -> Result<(), Error> {
                self.0.extend(Some((self.1)(Cast::Str(v))));

                Ok(())
            }

            fn none(&mut self) -> Result<(), Error> {
                self.0.extend(Some((self.1)(Cast::None)));

                Ok(())
            }

            #[cfg(feature = "error")]
            fn error(&mut self, _: &(dyn super::error::Error + 'static)) -> Result<(), Error> {
                self.0.extend(Some((self.1)(Cast::None)));

                Ok(())
            }

            #[cfg(feature = "sval2")]
            fn sval2(&mut self, v: &dyn super::sval::v2::Value) -> Result<(), Error> {
                super::sval::v2::internal_visit(v, self)
            }

            #[cfg(feature = "sval2")]
            fn borrowed_sval2(&mut self, v: &'v dyn super::sval::v2::Value) -> Result<(), Error> {
                super::sval::v2::borrowed_internal_visit(v, self)
            }

            #[cfg(feature = "serde1")]
            fn serde1(&mut self, v: &dyn super::serde::v1::Serialize) -> Result<(), Error> {
                super::serde::v1::internal_visit(v, self)
            }

            fn poisoned(&mut self, _: &'static str) -> Result<(), Error> {
                self.0.extend(Some((self.1)(Cast::None)));

                Ok(())
            }
        }

        let _ = self.internal_visit(&mut Visitor(collection, cast, PhantomData));
    }
}

pub(in crate::internal) enum Cast<'v> {
    Signed(i64),
    Unsigned(u64),
    BigSigned(i128),
    BigUnsigned(u128),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(&'v str),
    None,
    #[cfg(feature = "alloc")]
    String(String),
}

impl<'v> Cast<'v> {
    #[inline]
    fn into_owned(self) -> Option<Cast<'static>> {
        match self {
            Cast::Signed(v) => Some(Cast::Signed(v)),
            Cast::Unsigned(v) => Some(Cast::Unsigned(v)),
            Cast::BigSigned(v) => Some(Cast::BigSigned(v)),
            Cast::BigUnsigned(v) => Some(Cast::BigUnsigned(v)),
            Cast::Float(v) => Some(Cast::Float(v)),
            Cast::Bool(v) => Some(Cast::Bool(v)),
            Cast::Char(v) => Some(Cast::Char(v)),
            Cast::None => Some(Cast::None),
            #[cfg(feature = "alloc")]
            Cast::String(v) => Some(Cast::String(v)),
            #[cfg(feature = "alloc")]
            Cast::Str(v) => Some(Cast::String(v.into())),
            #[cfg(not(feature = "alloc"))]
            Cast::Str(_) => None,
        }
    }

    #[inline]
    fn into_borrowed_str(self) -> Option<&'v str> {
        if let Cast::Str(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    fn into_u64(self) -> Option<u64> {
        match self {
            Cast::Unsigned(value) => Some(value),
            Cast::BigUnsigned(value) => value.try_into().ok(),
            Cast::Signed(value) => value.try_into().ok(),
            Cast::BigSigned(value) => value.try_into().ok(),
            _ => None,
        }
    }

    #[inline]
    fn into_i64(self) -> Option<i64> {
        match self {
            Cast::Signed(value) => Some(value),
            Cast::BigSigned(value) => value.try_into().ok(),
            Cast::Unsigned(value) => value.try_into().ok(),
            Cast::BigUnsigned(value) => value.try_into().ok(),
            _ => None,
        }
    }

    #[inline]
    fn into_u128(self) -> Option<u128> {
        match self {
            Cast::BigUnsigned(value) => Some(value),
            Cast::Unsigned(value) => Some(value.into()),
            Cast::Signed(value) => value.try_into().ok(),
            Cast::BigSigned(value) => value.try_into().ok(),
            _ => None,
        }
    }

    #[inline]
    fn into_i128(self) -> Option<i128> {
        match self {
            Cast::BigSigned(value) => Some(value),
            Cast::Signed(value) => Some(value.into()),
            Cast::Unsigned(value) => value.try_into().ok(),
            Cast::BigUnsigned(value) => value.try_into().ok(),
            _ => None,
        }
    }

    #[inline]
    fn into_f64(self) -> Option<f64> {
        match self {
            Cast::Float(value) => Some(value),
            Cast::Unsigned(value) => u32::try_from(value)
                .ok()
                .and_then(|value| value.try_into().ok()),
            Cast::Signed(value) => i32::try_from(value)
                .ok()
                .and_then(|value| value.try_into().ok()),
            Cast::BigUnsigned(value) => u32::try_from(value)
                .ok()
                .and_then(|value| value.try_into().ok()),
            Cast::BigSigned(value) => i32::try_from(value)
                .ok()
                .and_then(|value| value.try_into().ok()),
            _ => None,
        }
    }

    #[inline]
    fn into_char(self) -> Option<char> {
        if let Cast::Char(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    fn into_bool(self) -> Option<bool> {
        if let Cast::Bool(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(feature = "alloc")]
mod alloc_support {
    use super::*;

    use crate::std::borrow::Cow;

    impl<'v> ValueBag<'v> {
        /// Try get a `str` from this value.
        ///
        /// This method is cheap for primitive types, but may call arbitrary
        /// serialization implementations for complex ones. If the serialization
        /// implementation produces a short lived string it will be allocated.
        #[inline]
        pub fn to_str(&self) -> Option<Cow<'v, str>> {
            self.inner.cast().into_str()
        }

        /// Try push nested values as `str`s from this value into the given collection.
        ///
        /// If this value is a primitive type then this method is equivalent to `to_str`.
        /// If this value is a sequence then each element will be cast to a `str`.
        /// Any elements that fail to cast will be passed as `None`s.
        pub fn collect_str(&self, into: &mut (impl Extend<Option<Cow<'v, str>>> + ?Sized)) {
            self.inner.collect(into, |cast| cast.into_str())
        }
    }

    impl<'v> Cast<'v> {
        #[inline]
        pub(in crate::internal) fn into_str(self) -> Option<Cow<'v, str>> {
            match self {
                Cast::Str(value) => Some(value.into()),
                Cast::String(value) => Some(value.into()),
                _ => None,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        #[cfg(target_arch = "wasm32")]
        use wasm_bindgen_test::*;

        use crate::{
            std::borrow::{Cow, ToOwned},
            test::IntoValueBag,
            ValueBag,
        };

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
                    .to_borrowed_str()
                    .expect("invalid value")
            );
            assert_eq!(
                "a string",
                ValueBag::try_capture(&short_lived)
                    .expect("invalid value")
                    .to_borrowed_str()
                    .expect("invalid value")
            );
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn primitive_collect() {
            use crate::std::vec::Vec;

            let mut vec = Vec::<Option<Cow<str>>>::new();
            "string".into_value_bag().collect_str(&mut vec);
            assert_eq!(vec![Some(Cow::Borrowed("string"))], vec);
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    use super::*;

    use crate::std::string::ToString;

    use crate::test::IntoValueBag;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn primitive_capture_str() {
        let s: &str = &"short lived".to_string();
        assert_eq!(
            "short lived",
            ValueBag::try_capture(s)
                .unwrap()
                .to_borrowed_str()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn primitive_cast() {
        assert_eq!(
            "a string",
            "a string"
                .into_value_bag()
                .by_ref()
                .to_borrowed_str()
                .expect("invalid value")
        );

        assert_eq!(
            1u64,
            1u8.into_value_bag()
                .by_ref()
                .to_u64()
                .expect("invalid value")
        );
        assert_eq!(
            1u64,
            1u16.into_value_bag()
                .by_ref()
                .to_u64()
                .expect("invalid value")
        );
        assert_eq!(
            1u64,
            1u32.into_value_bag()
                .by_ref()
                .to_u64()
                .expect("invalid value")
        );
        assert_eq!(
            1u64,
            1u64.into_value_bag()
                .by_ref()
                .to_u64()
                .expect("invalid value")
        );
        assert_eq!(
            1u64,
            1usize
                .into_value_bag()
                .by_ref()
                .to_u64()
                .expect("invalid value")
        );
        assert_eq!(
            1u128,
            1u128
                .into_value_bag()
                .by_ref()
                .to_u128()
                .expect("invalid value")
        );

        assert_eq!(
            -1i64,
            -1i8.into_value_bag()
                .by_ref()
                .to_i64()
                .expect("invalid value")
        );
        assert_eq!(
            -1i64,
            -1i8.into_value_bag()
                .by_ref()
                .to_i64()
                .expect("invalid value")
        );
        assert_eq!(
            -1i64,
            -1i8.into_value_bag()
                .by_ref()
                .to_i64()
                .expect("invalid value")
        );
        assert_eq!(
            -1i64,
            -1i64
                .into_value_bag()
                .by_ref()
                .to_i64()
                .expect("invalid value")
        );
        assert_eq!(
            -1i64,
            -1isize
                .into_value_bag()
                .by_ref()
                .to_i64()
                .expect("invalid value")
        );
        assert_eq!(
            -1i128,
            -1i128
                .into_value_bag()
                .by_ref()
                .to_i128()
                .expect("invalid value")
        );

        assert!(1f64.into_value_bag().by_ref().to_f64().is_some());
        assert!(1u64.into_value_bag().by_ref().to_f64().is_some());
        assert!((-1i64).into_value_bag().by_ref().to_f64().is_some());
        assert!(1u128.into_value_bag().by_ref().to_f64().is_some());
        assert!((-1i128).into_value_bag().by_ref().to_f64().is_some());

        assert!(u64::MAX.into_value_bag().by_ref().to_u128().is_some());
        assert!(i64::MIN.into_value_bag().by_ref().to_i128().is_some());
        assert!(i64::MAX.into_value_bag().by_ref().to_u64().is_some());

        assert!((-1i64).into_value_bag().by_ref().to_u64().is_none());
        assert!(u64::MAX.into_value_bag().by_ref().to_i64().is_none());
        assert!(u64::MAX.into_value_bag().by_ref().to_f64().is_none());

        assert!(i128::MAX.into_value_bag().by_ref().to_i64().is_none());
        assert!(u128::MAX.into_value_bag().by_ref().to_u64().is_none());

        assert!(1f64.into_value_bag().by_ref().to_u64().is_none());

        assert_eq!(
            'a',
            'a'.into_value_bag()
                .by_ref()
                .to_char()
                .expect("invalid value")
        );
        assert_eq!(
            true,
            true.into_value_bag()
                .by_ref()
                .to_bool()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn primitive_collect() {
        use crate::std::vec::Vec;

        let mut vec = Vec::<Option<u64>>::new();
        1u64.into_value_bag().collect_u64(&mut vec);
        assert_eq!(vec![Some(1u64)], vec);

        let mut vec = Vec::<Option<i64>>::new();
        1i64.into_value_bag().collect_i64(&mut vec);
        assert_eq!(vec![Some(1i64)], vec);

        let mut vec = Vec::<Option<u128>>::new();
        (&1u128).into_value_bag().collect_u128(&mut vec);
        assert_eq!(vec![Some(1u128)], vec);

        let mut vec = Vec::<Option<i128>>::new();
        (&1i128).into_value_bag().collect_i128(&mut vec);
        assert_eq!(vec![Some(1i128)], vec);

        let mut vec = Vec::<Option<f64>>::new();
        1f64.into_value_bag().collect_f64(&mut vec);
        assert_eq!(vec![Some(1f64)], vec);

        let mut vec = Vec::<Option<bool>>::new();
        true.into_value_bag().collect_bool(&mut vec);
        assert_eq!(vec![Some(true)], vec);

        let mut vec = Vec::<Option<char>>::new();
        'a'.into_value_bag().collect_char(&mut vec);
        assert_eq!(vec![Some('a')], vec);

        let mut vec = Vec::<Option<&str>>::new();
        "string".into_value_bag().collect_borrowed_str(&mut vec);
        assert_eq!(vec![Some("string")], vec);
    }
}
