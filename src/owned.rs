use crate::{internal, ValueBag};

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

impl OwnedValueBag {
    /// Get a regular [`ValueBag`] from this type.
    ///
    /// Once a `ValueBag` has been buffered, it will behave
    /// slightly differently when converted back:
    ///
    /// - `fmt::Debug` won't use formatting flags.
    /// - `serde::Serialize` will use the text-based representation.
    /// - The original type will change, so downcasting won't work.
    pub fn by_ref<'v>(&'v self) -> ValueBag<'v> {
        ValueBag {
            inner: self.inner.by_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::std::mem;

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
}
