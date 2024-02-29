use crate::{
    fill::Fill,
    internal::{self, Internal},
    std::sync::Arc,
    ValueBag,
};

/// A dynamic structured value.
///
/// This type is an owned variant of [`ValueBag`] that can be
/// constructed using its [`to_owned`](struct.ValueBag.html#method.to_owned) method.
/// `OwnedValueBag`s are suitable for storing and sharing across threads.
///
/// `OwnedValueBag`s can be inspected by converting back into a regular `ValueBag`
/// using the [`by_ref`](#method.by_ref) method.
#[derive(Clone)]
pub struct OwnedValueBag {
    inner: internal::owned::OwnedInternal,
}

impl<'v> ValueBag<'v> {
    /// Buffer this value into an [`OwnedValueBag`].
    pub fn to_owned(&self) -> OwnedValueBag {
        OwnedValueBag {
            inner: self.inner.to_owned(),
        }
    }
}

impl ValueBag<'static> {
    /// Get a value from an owned, sharable, debuggable type.
    ///
    /// This method will attempt to capture the given value as a well-known primitive
    /// before resorting to using its `Debug` implementation.
    pub fn capture_owned_debug<T>(value: T) -> Self
    where
        T: internal::fmt::Debug + Send + Sync + 'static,
    {
        Self::try_capture_owned(&value).unwrap_or_else(|| ValueBag {
            inner: Internal::SharedDebug(Arc::new(value)),
        })
    }

    /// Get a value from an owned, sharable, displayable type.
    ///
    /// This method will attempt to capture the given value as a well-known primitive
    /// before resorting to using its `Display` implementation.
    pub fn capture_owned_display<T>(value: T) -> Self
    where
        T: internal::fmt::Display + Send + Sync + 'static,
    {
        Self::try_capture_owned(&value).unwrap_or_else(|| ValueBag {
            inner: Internal::SharedDisplay(Arc::new(value)),
        })
    }

    /// Get a value from an owned, shared, fillable slot.
    pub fn capture_owned_fill<T>(value: T) -> Self
    where
        T: Fill + Send + Sync + 'static,
    {
        ValueBag {
            inner: Internal::SharedFill(Arc::new(value)),
        }
    }

    /// Get a value from an owned, shared error.
    #[cfg(feature = "error")]
    pub fn capture_owned_error<T>(value: T) -> Self
    where
        T: internal::error::Error + Send + Sync + 'static,
    {
        ValueBag {
            inner: Internal::SharedError(Arc::new(value)),
        }
    }

    /// Get a value from an owned, shared, structured type.
    ///
    /// This method will attempt to capture the given value as a well-known primitive
    /// before resorting to using its `Value` implementation.
    #[cfg(feature = "sval2")]
    pub fn capture_owned_sval2<T>(value: T) -> Self
    where
        T: value_bag_sval2::lib::Value + Send + Sync + 'static,
    {
        Self::try_capture_owned(&value).unwrap_or(ValueBag {
            inner: Internal::SharedSval2(Arc::new(value)),
        })
    }

    /// Get a value from an owned, shared, structured type.
    ///
    /// This method will attempt to capture the given value as a well-known primitive
    /// before resorting to using its `Value` implementation.
    #[cfg(feature = "serde1")]
    pub fn capture_owned_serde1<T>(value: T) -> Self
    where
        T: value_bag_serde1::lib::Serialize + Send + Sync + 'static,
    {
        Self::try_capture_owned(&value).unwrap_or(ValueBag {
            inner: Internal::SharedSerde1(Arc::new(value)),
        })
    }
}

impl OwnedValueBag {
    /// Get a regular [`ValueBag`] from this type.
    ///
    /// Once a `ValueBag` has been buffered, it will behave
    /// slightly differently when converted back:
    ///
    /// - `fmt::Debug` won't use formatting flags.
    /// - `serde::Serialize` will use the text-based representation.
    /// - The original type will change, so downcasting won't work.
    pub const fn by_ref<'v>(&'v self) -> ValueBag<'v> {
        ValueBag {
            inner: self.inner.by_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    use super::*;

    use crate::{
        fill,
        std::{mem, string::ToString},
    };

    const SIZE_LIMIT_U64: usize = 4;

    #[test]
    fn is_send_sync() {
        fn assert<T: Send + Sync + 'static>() {}

        assert::<OwnedValueBag>();
    }

    #[test]
    fn owned_value_bag_size() {
        let size = mem::size_of::<OwnedValueBag>();
        let limit = mem::size_of::<u64>() * SIZE_LIMIT_U64;

        if size > limit {
            panic!(
                "`OwnedValueBag` size ({} bytes) is too large (expected up to {} bytes)",
                size, limit,
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn fill_to_owned() {
        let value = ValueBag::from_fill(&|slot: fill::Slot| slot.fill_any(42u64)).to_owned();

        assert!(matches!(
            value.inner,
            internal::owned::OwnedInternal::BigUnsigned(42)
        ));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn owned_fill_to_owned() {
        let value = ValueBag::capture_owned_fill(|slot: fill::Slot| slot.fill_any(42u64)).to_owned();

        assert!(matches!(
            value.inner,
            internal::owned::OwnedInternal::SharedFill(_),
        ));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn fmt_to_owned() {
        let debug = ValueBag::from_debug(&"a value").to_owned();
        let display = ValueBag::from_display(&"a value").to_owned();

        assert!(matches!(
            debug.inner,
            internal::owned::OwnedInternal::Debug(_)
        ));
        assert!(matches!(
            display.inner,
            internal::owned::OwnedInternal::Display(_)
        ));

        assert_eq!("\"a value\"", debug.to_string());
        assert_eq!("a value", display.to_string());

        let debug = debug.by_ref();
        let display = display.by_ref();

        assert!(matches!(debug.inner, internal::Internal::AnonDebug(_)));
        assert!(matches!(display.inner, internal::Internal::AnonDisplay(_)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn owned_fmt_to_owned() {
        let debug = ValueBag::capture_owned_debug("a value".to_string()).to_owned();
        let display = ValueBag::capture_owned_display("a value".to_string()).to_owned();

        assert!(matches!(
            debug.inner,
            internal::owned::OwnedInternal::SharedDebug(_)
        ));
        assert!(matches!(
            display.inner,
            internal::owned::OwnedInternal::SharedDisplay(_)
        ));

        assert_eq!("\"a value\"", debug.to_string());
        assert_eq!("a value", display.to_string());

        let debug = debug.by_ref();
        let display = display.by_ref();

        assert!(matches!(debug.inner, internal::Internal::SharedRefDebug(_)));
        assert!(matches!(display.inner, internal::Internal::SharedRefDisplay(_)));
    }

    #[test]
    #[cfg(feature = "error")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn error_to_owned() {
        use crate::std::io;

        let value =
            ValueBag::from_dyn_error(&io::Error::new(io::ErrorKind::Other, "something failed!"))
                .to_owned();

        assert!(matches!(
            value.inner,
            internal::owned::OwnedInternal::Error(_)
        ));

        let value = value.by_ref();

        assert!(matches!(value.inner, internal::Internal::AnonError(_)));

        assert!(value.to_borrowed_error().is_some());
    }

    #[test]
    #[cfg(feature = "error")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn owned_error_to_owned() {
        use crate::std::io;

        let value =
            ValueBag::capture_owned_error(io::Error::new(io::ErrorKind::Other, "something failed!"))
                .to_owned();

        assert!(matches!(
            value.inner,
            internal::owned::OwnedInternal::SharedError(_)
        ));

        let value = value.by_ref();

        assert!(matches!(value.inner, internal::Internal::SharedRefError(_)));
    }

    #[test]
    #[cfg(feature = "serde1")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn serde1_to_owned() {
        let value = ValueBag::from_serde1(&42u64).to_owned();

        assert!(matches!(
            value.inner,
            internal::owned::OwnedInternal::Serde1(_)
        ));

        let value = value.by_ref();

        assert!(matches!(value.inner, internal::Internal::AnonSerde1(_)));
    }

    #[test]
    #[cfg(feature = "serde1")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn owned_serde1_to_owned() {
        let value = ValueBag::capture_owned_serde1("a value".to_string()).to_owned();

        assert!(matches!(
            value.inner,
            internal::owned::OwnedInternal::SharedSerde1(_)
        ));

        let value = value.by_ref();

        assert!(matches!(value.inner, internal::Internal::SharedRefSerde1(_)));
    }

    #[test]
    #[cfg(feature = "sval2")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_to_owned() {
        let value = ValueBag::from_sval2(&42u64).to_owned();

        assert!(matches!(
            value.inner,
            internal::owned::OwnedInternal::Sval2(_)
        ));

        let value = value.by_ref();

        assert!(matches!(value.inner, internal::Internal::AnonSval2(_)));
    }

    #[test]
    #[cfg(feature = "sval2")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn owned_sval2_to_owned() {
        let value = ValueBag::capture_owned_sval2("a value".to_string()).to_owned();

        assert!(matches!(
            value.inner,
            internal::owned::OwnedInternal::SharedSval2(_)
        ));

        let value = value.by_ref();

        assert!(matches!(value.inner, internal::Internal::SharedRefSval2(_)));
    }
}
