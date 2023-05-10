use crate::{internal, ValueBag};

#[derive(Clone)]
pub struct OwnedValueBag {
    inner: internal::owned::OwnedInternal,
}

impl<'v> ValueBag<'v> {
    pub fn to_owned(&self) -> OwnedValueBag {
        OwnedValueBag {
            inner: self.inner.to_owned(),
        }
    }
}

impl OwnedValueBag {
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
