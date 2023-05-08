use crate::{ValueBag, internal};

pub struct OwnedValueBag {
    inner: internal::OwnedInternal,
}

impl<'v> ValueBag<'v> {
    pub fn to_owned(&self) -> OwnedValueBag {
        todo!()
    }
}

impl OwnedValueBag {
    pub fn to_value<'v>(&'v self) -> ValueBag<'v> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn is_send_sync() {
        fn assert<T: Send + Sync + 'static>() {}
        
        assert::<OwnedValueBag>();
    }
}
