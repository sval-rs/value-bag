use crate::{
    fill::{Fill, Slot},
    internal::{seq, Internal},
    std::marker::PhantomData,
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
        self.inner
            .extend::<seq::ExtendPrimitive<S, u64>>()
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
            .extend::<seq::ExtendPrimitive<S, i64>>()
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
            .extend::<seq::ExtendPrimitive<S, u128>>()
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
            .extend::<seq::ExtendPrimitive<S, i128>>()
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
            .extend::<seq::ExtendPrimitive<S, f64>>()
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

        impl<'a, S: Extend<f64>> seq::ExtendValue<'a> for ExtendF64<S> {
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
            .extend::<seq::ExtendPrimitive<S, bool>>()
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

        impl<'a, S: Extend<Option<&'a str>>> seq::ExtendValue<'a> for ExtendStr<'a, S> {
            fn extend<'b>(&mut self, inner: Internal<'b>) {
                self.0.extend(Some(None::<&'a str>))
            }

            fn extend_borrowed(&mut self, inner: Internal<'a>) {
                self.0.extend(Some(ValueBag { inner }.to_borrowed_str()))
            }
        }

        self.inner.extend::<ExtendStr<'v, S>>().map(|seq| seq.0)
    }
}

#[repr(transparent)]
struct FillPrimitive<T>(T);

impl<T> FillPrimitive<T> {
    pub fn new_ref<'a>(v: &'a T) -> &'a FillPrimitive<T> {
        // SAFETY: `FillPrimitive<T>` and `T` have the same ABI
        unsafe { &*(v as *const T as *const FillPrimitive<T>) }
    }
}

impl<T, const N: usize> Fill for FillPrimitive<[T; N]>
where
    for<'v> &'v T: Into<ValueBag<'v>>,
{
    fn fill(&self, slot: Slot) -> Result<(), Error> {
        slot.fill(|visitor| visitor.seq(&|| self.0.iter().map(|v| v.into().inner)))
    }
}

impl<'v, 'a, T> Fill for FillPrimitive<&'v [T]>
where
    &'v T: Into<ValueBag<'v>>,
{
    fn fill(&self, slot: Slot) -> Result<(), Error> {
        slot.fill(|visitor| visitor.seq(&|| self.0.iter().map(|v| v.into().inner)))
    }
}

#[repr(transparent)]
struct FillRef<T>(T);

impl<T> FillRef<T> {
    pub fn new_ref<'a>(v: &'a T) -> &'a FillRef<T> {
        // SAFETY: `FillRef<T>` and `T` have the same ABI
        unsafe { &*(v as *const T as *const FillRef<T>) }
    }
}

impl<'v, 'a, T: ?Sized, const N: usize> Fill for FillRef<[&'a T; N]>
where
    &'a T: Into<ValueBag<'a>>,
{
    fn fill(&self, slot: Slot) -> Result<(), Error> {
        slot.fill(|visitor| visitor.seq(&|| self.0.iter().map(|v| (*v).into().inner)))
    }

    fn fill_borrowed<'b>(&'b self, slot: Slot<'_, 'b>) -> Result<(), Error> {
        slot.fill(|visitor| visitor.borrowed_seq(&|| self.0.iter().map(|v| (*v).into().inner)))
    }
}

impl<'v, T: ?Sized> Fill for FillRef<&'v [&'v T]>
where
    &'v T: Into<ValueBag<'v>>,
{
    fn fill(&self, slot: Slot) -> Result<(), Error> {
        slot.fill(|visitor| visitor.seq(&|| self.0.iter().map(|v| (*v).into().inner)))
    }

    fn fill_borrowed<'a>(&'a self, slot: Slot<'_, 'a>) -> Result<(), Error> {
        slot.fill(|visitor| visitor.borrowed_seq(&|| self.0.iter().map(|v| (*v).into().inner)))
    }
}

macro_rules! convert_primitive(
    ($($t:ty,)*) => {
        $(
            impl<'v, const N: usize> From<&'v [$t; N]> for ValueBag<'v> {
                fn from(v: &'v [$t; N]) -> Self {
                    ValueBag::from_fill(FillPrimitive::new_ref(v))
                }
            }

            impl<'v, const N: usize> From<Option<&'v [$t; N]>> for ValueBag<'v> {
                fn from(v: Option<&'v [$t; N]>) -> Self {
                    ValueBag::from_option(v)
                }
            }

            impl<'a, 'v> From<&'v &'a [$t]> for ValueBag<'v> {
                fn from(v: &'v &'a [$t]) -> Self {
                    ValueBag::from_fill(FillPrimitive::new_ref(v))
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

impl<'v, const N: usize> TryFrom<ValueBag<'v>> for [u8; N] {
    type Error = Error;

    fn try_from(v: ValueBag<'v>) -> Result<[u8; N], Error> {
        todo!()
    }
}

impl<'v, 'a, const N: usize> From<&'v [&'a str; N]> for ValueBag<'v> {
    fn from(v: &'v [&'a str; N]) -> Self {
        ValueBag::from_fill(FillRef::new_ref(v))
    }
}

impl<'v, 'a, const N: usize> From<Option<&'v [&'a str; N]>> for ValueBag<'v> {
    fn from(v: Option<&'v [&'a str; N]>) -> Self {
        ValueBag::from_option(v)
    }
}

impl<'v, 'a, 'b> From<&'v &'a [&'b str]> for ValueBag<'v> {
    fn from(v: &'v &'a [&'b str]) -> Self {
        ValueBag::from_fill(FillRef::new_ref(v))
    }
}

impl<'v, 'a, 'b> From<Option<&'v &'a [&'b str]>> for ValueBag<'v> {
    fn from(v: Option<&'v &'a [&'b str]>) -> Self {
        ValueBag::from_option(v)
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

            impl<'a, S: Extend<Option<Cow<'a, str>>>> seq::ExtendValue<'a> for ExtendStr<'a, S> {
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
            v.to_borrowed_str_seq::<Vec<Option<&str>>>()
        );
    }
}
