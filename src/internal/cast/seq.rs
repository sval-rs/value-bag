use crate::std::{fmt, marker::PhantomData};

use crate::{
    internal::{self, Internal, InternalVisitor},
    Error, ValueBag,
};

impl<'v> ValueBag<'v> {
    /// Try get a collection `S` of `u64`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_u64_seq<S: Default + Extend<Option<u64>>>(&self) -> Option<S> {
        self.inner.seq::<ExtendCast<S, u64>>().map(|seq| seq.0)
    }

    /// Try get a collection `S` of `i64`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_i64_seq<S: Default + Extend<Option<i64>>>(&self) -> Option<S> {
        self.inner.seq::<ExtendCast<S, i64>>().map(|seq| seq.0)
    }

    /// Try get a collection `S` of `u128`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_u128_seq<S: Default + Extend<Option<u128>>>(&self) -> Option<S> {
        self.inner.seq::<ExtendCast<S, u128>>().map(|seq| seq.0)
    }

    /// Try get a collection `S` of `i128`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_i128_seq<S: Default + Extend<Option<i128>>>(&self) -> Option<S> {
        self.inner.seq::<ExtendCast<S, i128>>().map(|seq| seq.0)
    }

    /// Try get a collection `S` of `f64`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_f64_seq<S: Default + Extend<Option<f64>>>(&self) -> Option<S> {
        self.inner.seq::<ExtendCast<S, f64>>().map(|seq| seq.0)
    }

    /// Try get a collection `S` of `bool`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_bool_seq<S: Default + Extend<Option<bool>>>(&self) -> Option<S> {
        self.inner.seq::<ExtendCast<S, bool>>().map(|seq| seq.0)
    }
}

#[derive(Default)]
struct ExtendCast<S, T>(S, PhantomData<T>);

impl<'a, S: Extend<Option<T>>, T: TryFrom<ValueBag<'a>>> Extend<Internal<'a>> for ExtendCast<S, T> {
    fn extend<I: IntoIterator<Item = Internal<'a>>>(&mut self, iter: I) {
        self.0.extend(
            iter.into_iter()
                .map(|inner| ValueBag { inner }.try_into().ok()),
        )
    }
}

impl<'v> Internal<'v> {
    #[inline]
    fn seq<S: Default + for<'a> Extend<Internal<'a>>>(&self) -> Option<S> {
        struct SeqVisitor<S>(Option<S>);

        impl<'v, S: Default + for<'a> Extend<Internal<'a>>> InternalVisitor<'v> for SeqVisitor<S> {
            #[inline]
            fn debug(&mut self, _: &dyn fmt::Debug) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn display(&mut self, _: &dyn fmt::Display) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn u64(&mut self, _: u64) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn i64(&mut self, _: i64) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn u128(&mut self, _: &u128) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn i128(&mut self, _: &i128) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn f64(&mut self, _: f64) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn bool(&mut self, _: bool) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn char(&mut self, _: char) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn str(&mut self, _: &str) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn none(&mut self) -> Result<(), Error> {
                Ok(())
            }

            #[cfg(feature = "error")]
            #[inline]
            fn error(&mut self, _: &dyn internal::error::Error) -> Result<(), Error> {
                Ok(())
            }

            #[cfg(feature = "sval2")]
            #[inline]
            fn sval2(&mut self, v: &dyn internal::sval::v2::Value) -> Result<(), Error> {
                self.0 = internal::sval::v2::seq(v);

                Ok(())
            }

            #[cfg(feature = "serde1")]
            #[inline]
            fn serde1(&mut self, v: &dyn internal::serde::v1::Serialize) -> Result<(), Error> {
                self.0 = internal::serde::v1::seq(v);

                Ok(())
            }

            fn poisoned(&mut self, _: &'static str) -> Result<(), Error> {
                Ok(())
            }
        }

        let mut visitor = SeqVisitor(None);
        let _ = self.internal_visit(&mut visitor);

        visitor.0
    }
}
