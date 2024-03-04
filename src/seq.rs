use crate::{
    internal::{seq, Internal},
    fill::Slot,
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
        self.inner.extend::<seq::ExtendPrimitive<S, u64>>().map(|seq| seq.into_inner())
    }

    /// Try get a collection `S` of `i64`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_i64_seq<S: Default + Extend<Option<i64>>>(&self) -> Option<S> {
        self.inner.extend::<seq::ExtendPrimitive<S, i64>>().map(|seq| seq.into_inner())
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
        self.inner.extend::<seq::ExtendPrimitive<S, f64>>().map(|seq| seq.into_inner())
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
}

impl<'s, 'f> Slot<'s, 'f> {
    /// Fill the slot with a value.
    ///
    /// The given value doesn't need to satisfy any particular lifetime constraints.
    pub fn fill_seq<F, I, T>(self, value: F) -> Result<(), Error>
    where
        F: Fn() -> I,
        I: Iterator<Item = T>,
        T: Into<ValueBag<'f>>,
    {
        self.fill(|visitor| visitor.borrowed_seq(&move || value().map(|v| v.into().inner)))
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

            self.inner.extend::<ExtendStr<'v, S>>().map(|seq| seq.into_inner())
        }
    }
}
