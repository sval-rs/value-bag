use crate::{
    fill::Slot,
    internal::{Internal, InternalVisitor},
    std::{any::Any, fmt, marker::PhantomData, mem, ops::ControlFlow},
    Error, ValueBag,
};

impl<'v> ValueBag<'v> {
    /// Get a value from a sequence of values without capturing support.
    pub fn from_seq_slice<I, T>(value: &'v I) -> Self
    where
        I: AsRef<[T]>,
        &'v T: Into<ValueBag<'v>> + 'v,
    {
        ValueBag {
            inner: Internal::AnonSeq(SeqSlice::new_ref(value)),
        }
    }

    pub(crate) const fn from_dyn_seq(value: &'v dyn Seq) -> Self {
        ValueBag {
            inner: Internal::AnonSeq(value),
        }
    }

    /// Try get a collection `S` of `u64`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_u64_seq<S: Default + Extend<Option<u64>>>(&self) -> Option<S> {
        self.inner
            .extend::<ExtendPrimitive<S, u64>>()
            .map(|seq| seq.into_inner())
    }

    /// Try get a collection `S` of `i64`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_i64_seq<S: Default + Extend<Option<i64>>>(&self) -> Option<S> {
        self.inner
            .extend::<ExtendPrimitive<S, i64>>()
            .map(|seq| seq.into_inner())
    }

    /// Try get a collection `S` of `u128`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_u128_seq<S: Default + Extend<Option<u128>>>(&self) -> Option<S> {
        self.inner
            .extend::<ExtendPrimitive<S, u128>>()
            .map(|seq| seq.into_inner())
    }

    /// Try get a collection `S` of `i128`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_i128_seq<S: Default + Extend<Option<i128>>>(&self) -> Option<S> {
        self.inner
            .extend::<ExtendPrimitive<S, i128>>()
            .map(|seq| seq.into_inner())
    }

    /// Try get a collection `S` of `f64`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_f64_seq<S: Default + Extend<Option<f64>>>(&self) -> Option<S> {
        self.inner
            .extend::<ExtendPrimitive<S, f64>>()
            .map(|seq| seq.into_inner())
    }

    /// Get a collection `S` of `f64`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the conversion of each of its elements. The conversion is the
    /// same as [`ValueBag::as_f64`].
    ///
    /// If this value is not a sequence then this method will return an
    /// empty collection.
    ///
    /// This is similar to [`ValueBag::to_f64_seq`], but can be more
    /// convenient when there's no need to distinguish between an empty
    /// collection and a non-collection, or between `f64` and non-`f64` elements.
    pub fn as_f64_seq<S: Default + Extend<f64>>(&self) -> S {
        #[derive(Default)]
        struct ExtendF64<S>(S);

        impl<'a, S: Extend<f64>> ExtendValue<'a> for ExtendF64<S> {
            fn extend<'b>(&mut self, inner: Internal<'b>) {
                self.0.extend(Some(ValueBag { inner }.as_f64()))
            }
        }

        self.inner
            .extend::<ExtendF64<S>>()
            .map(|seq| seq.0)
            .unwrap_or_default()
    }

    /// Try get a collection `S` of `bool`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_bool_seq<S: Default + Extend<Option<bool>>>(&self) -> Option<S> {
        self.inner
            .extend::<ExtendPrimitive<S, bool>>()
            .map(|seq| seq.into_inner())
    }

    /// Try get a collection `S` of strings from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    #[inline]
    pub fn to_borrowed_str_seq<S: Default + Extend<Option<&'v str>>>(&self) -> Option<S> {
        #[derive(Default)]
        struct ExtendStr<'a, S>(S, PhantomData<&'a str>);

        impl<'a, S: Extend<Option<&'a str>>> ExtendValue<'a> for ExtendStr<'a, S> {
            fn extend<'b>(&mut self, _: Internal<'b>) {
                self.0.extend(Some(None::<&'a str>))
            }

            fn extend_borrowed(&mut self, inner: Internal<'a>) {
                self.0.extend(Some(ValueBag { inner }.to_borrowed_str()))
            }
        }

        self.inner.extend::<ExtendStr<'v, S>>().map(|seq| seq.0)
    }
}

impl<'s, 'f> Slot<'s, 'f> {
    /// Fill the slot with a sequence of values.
    ///
    /// The given value doesn't need to satisfy any particular lifetime constraints.
    pub fn fill_seq_slice<I, T>(self, value: &'f I) -> Result<(), Error>
    where
        I: AsRef<[T]>,
        &'f T: Into<ValueBag<'f>> + 'f,
    {
        self.fill(|visitor| visitor.seq(SeqSlice::new_ref(value)))
    }
}

/*
This is a bit of an ugly way of working around the gulf between
lifetimes expressed externally as bounds, and lifetimes implied
on methods.
*/

#[repr(transparent)]
struct SeqSlice<'a, I: ?Sized, T>(PhantomData<&'a [T]>, I);

impl<'a, I: AsRef<[T]> + ?Sized + 'a, T> SeqSlice<'a, I, T> {
    fn new_ref(v: &'a I) -> &'a SeqSlice<'a, I, T> {
        // SAFETY: `SeqSlice<'a, I, T>` and `I` have the same ABI
        unsafe { &*(v as *const I as *const SeqSlice<'a, I, T>) }
    }

    fn as_ref<'b>(&'b self) -> &'a [T] {
        // SAFETY: `new_ref` requires there's a borrow of `&'a I`
        // on the borrow stack, so we can safely borrow it for `'a` here
        let inner = unsafe { mem::transmute::<&'b I, &'a I>(&self.1) };

        inner.as_ref()
    }
}

impl<'a, I, T> Seq for SeqSlice<'a, I, T>
where
    I: AsRef<[T]> + ?Sized + 'a,
    &'a T: Into<ValueBag<'a>>,
{
    fn for_each<'v>(&'v self, f: &mut dyn FnMut(Internal<'v>) -> ControlFlow<()>) {
        for v in self.as_ref().iter() {
            if let ControlFlow::Break(()) = f(v.into().inner) {
                return;
            }
        }
    }
}

pub(crate) trait Seq {
    fn for_each<'v>(&'v self, f: &mut dyn FnMut(Internal<'v>) -> ControlFlow<()>);
}

impl<'a, S: Seq + ?Sized> Seq for &'a S {
    fn for_each<'v>(&'v self, f: &mut dyn FnMut(Internal<'v>) -> ControlFlow<()>) {
        (**self).for_each(f)
    }
}

pub(crate) trait DowncastSeq {
    fn as_any(&self) -> &dyn Any;
    fn as_super(&self) -> &dyn Seq;
}

impl<T: Seq + 'static> DowncastSeq for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_super(&self) -> &dyn Seq {
        self
    }
}

impl<'a> Seq for dyn DowncastSeq + Send + Sync + 'a {
    fn for_each<'v>(&'v self, f: &mut dyn FnMut(Internal<'v>) -> ControlFlow<()>) {
        self.as_super().for_each(f)
    }
}

macro_rules! convert_primitive(
    ($($t:ty,)*) => {
        $(
            impl<'v, const N: usize> From<&'v [$t; N]> for ValueBag<'v> {
                fn from(v: &'v [$t; N]) -> Self {
                    ValueBag::from_seq_slice(v)
                }
            }

            impl<'v, const N: usize> From<Option<&'v [$t; N]>> for ValueBag<'v> {
                fn from(v: Option<&'v [$t; N]>) -> Self {
                    ValueBag::from_option(v)
                }
            }

            impl<'a, 'v> From<&'v &'a [$t]> for ValueBag<'v> {
                fn from(v: &'v &'a [$t]) -> Self {
                    ValueBag::from_seq_slice(v)
                }
            }

            impl<'a, 'v> From<Option<&'v &'a [$t]>> for ValueBag<'v> {
                fn from(v: Option<&'v &'a [$t]>) -> Self {
                    ValueBag::from_option(v)
                }
            }
        )*
    }
);

convert_primitive![
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, bool, char,
];

impl<'v, 'a, const N: usize> From<&'v [&'a str; N]> for ValueBag<'v> {
    fn from(v: &'v [&'a str; N]) -> Self {
        ValueBag::from_seq_slice(v)
    }
}

impl<'v, 'a, const N: usize> From<Option<&'v [&'a str; N]>> for ValueBag<'v> {
    fn from(v: Option<&'v [&'a str; N]>) -> Self {
        ValueBag::from_option(v)
    }
}

impl<'v, 'a, 'b> From<&'v &'a [&'b str]> for ValueBag<'v> {
    fn from(v: &'v &'a [&'b str]) -> Self {
        ValueBag::from_seq_slice(v)
    }
}

impl<'v, 'a, 'b> From<Option<&'v &'a [&'b str]>> for ValueBag<'v> {
    fn from(v: Option<&'v &'a [&'b str]>) -> Self {
        ValueBag::from_option(v)
    }
}

pub(crate) fn for_each_continue() -> ControlFlow<()> {
    ControlFlow::Continue(())
}

#[derive(Default)]
pub(crate) struct ExtendPrimitive<S, T>(S, PhantomData<T>);

impl<S, T> ExtendPrimitive<S, T> {
    pub fn into_inner(self) -> S {
        self.0
    }
}

impl<'a, S: Extend<Option<T>>, T: for<'b> TryFrom<ValueBag<'b>>> ExtendValue<'a>
    for ExtendPrimitive<S, T>
{
    fn extend<'b>(&mut self, inner: Internal<'b>) {
        self.0.extend(Some(ValueBag { inner }.try_into().ok()))
    }
}

#[allow(dead_code)]
pub(crate) trait ExtendValue<'v> {
    fn extend<'a>(&mut self, v: Internal<'a>);

    fn extend_borrowed(&mut self, v: Internal<'v>) {
        self.extend(v);
    }
}

impl<'v> Internal<'v> {
    #[inline]
    pub(crate) fn extend<S: Default + ExtendValue<'v>>(&self) -> Option<S> {
        struct SeqVisitor<S>(Option<S>);

        impl<'v, S: Default + ExtendValue<'v>> InternalVisitor<'v> for SeqVisitor<S> {
            #[inline]
            fn fill(&mut self, v: &dyn crate::fill::Fill) -> Result<(), Error> {
                v.fill(crate::fill::Slot::new(self))
            }

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
            fn error(&mut self, _: &dyn crate::internal::error::Error) -> Result<(), Error> {
                Ok(())
            }

            #[cfg(feature = "sval2")]
            #[inline]
            fn sval2(&mut self, v: &dyn crate::internal::sval::v2::Value) -> Result<(), Error> {
                self.0 = crate::internal::sval::v2::seq::extend(v);

                Ok(())
            }

            #[cfg(feature = "sval2")]
            #[inline]
            fn borrowed_sval2(
                &mut self,
                v: &'v dyn crate::internal::sval::v2::Value,
            ) -> Result<(), Error> {
                self.0 = crate::internal::sval::v2::seq::extend_borrowed(v);

                Ok(())
            }

            #[cfg(feature = "serde1")]
            #[inline]
            fn serde1(
                &mut self,
                v: &dyn crate::internal::serde::v1::Serialize,
            ) -> Result<(), Error> {
                self.0 = crate::internal::serde::v1::seq::extend(v);

                Ok(())
            }

            fn seq(&mut self, seq: &dyn Seq) -> Result<(), Error> {
                let mut s = S::default();

                seq.for_each(&mut |v| {
                    s.extend(v);
                    for_each_continue()
                });

                self.0 = Some(s);

                Ok(())
            }

            fn borrowed_seq(&mut self, seq: &'v dyn Seq) -> Result<(), Error> {
                let mut s = S::default();

                seq.for_each(&mut |v| {
                    s.extend_borrowed(v);
                    for_each_continue()
                });

                self.0 = Some(s);

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

#[cfg(feature = "alloc")]
mod alloc_support {
    use super::*;

    use crate::std::borrow::Cow;

    impl<'v> ValueBag<'v> {
        /// Try get a collection `S` of strings from this value.
        ///
        /// If this value is a sequence then the collection `S` will be extended
        /// with the attempted conversion of each of its elements.
        ///
        /// If this value is not a sequence then this method will return `None`.
        #[inline]
        pub fn to_str_seq<S: Default + Extend<Option<Cow<'v, str>>>>(&self) -> Option<S> {
            #[derive(Default)]
            struct ExtendStr<'a, S>(S, PhantomData<Cow<'a, str>>);

            impl<'a, S: Extend<Option<Cow<'a, str>>>> ExtendValue<'a> for ExtendStr<'a, S> {
                fn extend<'b>(&mut self, inner: Internal<'b>) {
                    self.0.extend(Some(
                        ValueBag { inner }
                            .to_str()
                            .map(|s| Cow::Owned(s.into_owned())),
                    ))
                }

                fn extend_borrowed(&mut self, inner: Internal<'a>) {
                    self.0.extend(Some(ValueBag { inner }.to_str()))
                }
            }

            self.inner.extend::<ExtendStr<'v, S>>().map(|seq| seq.0)
        }
    }
}

#[cfg(feature = "owned")]
pub(crate) mod owned {
    use super::*;

    use crate::{
        owned::OwnedValueBag,
        std::{boxed::Box, vec::Vec},
    };

    #[derive(Clone)]
    pub(crate) struct OwnedSeq(Box<[OwnedValueBag]>);

    impl Seq for OwnedSeq {
        fn for_each<'v>(&'v self, f: &mut dyn FnMut(Internal<'v>) -> ControlFlow<()>) {
            for item in self.0.iter() {
                if let ControlFlow::Break(()) = f(item.by_ref().inner) {
                    return;
                }
            }
        }
    }

    pub(crate) fn buffer(v: &dyn Seq) -> Result<OwnedSeq, Error> {
        let mut buf = Vec::new();

        v.for_each(&mut |inner| {
            buf.push(ValueBag { inner }.to_owned());
            ControlFlow::Continue(())
        });

        Ok(OwnedSeq(buf.into_boxed_slice()))
    }
}

#[cfg(test)]
mod tests {
    use std::vec::Vec;

    use super::*;

    #[test]
    fn to_borrowed_str_seq() {
        let v = ["a", "b", "c"];

        let v = ValueBag::from(&v);

        assert_eq!(
            Some(vec![Some("a"), Some("b"), Some("c")]),
            v.to_borrowed_str_<Vec<Option<&str>>>()
        );
    }
}
