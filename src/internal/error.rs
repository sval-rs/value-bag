use crate::{
    fill::Slot,
    std::error,
    ValueBag,
};

use super::{cast, Internal};

impl<'v> ValueBag<'v> {
    /// Get a value from an error.
    pub fn capture_error<T>(value: &'v T) -> Self
    where
        T: error::Error + 'static,
    {
        ValueBag {
            inner: Internal::Error {
                value,
                type_id: cast::type_id::<T>(),
            },
        }
    }

    /// Get a value from an erased value.
    pub fn from_dyn_error(value: &'v (dyn error::Error + 'static)) -> Self {
        ValueBag {
            inner: Internal::AnonError {
                value,
            }
        }
    }

    /// Try get an error from this value.
    pub fn to_borrowed_error(&self) -> Option<&(dyn Error + 'static)> {
        match self.inner {
            Internal::Error { value, .. } => Some(value),
            _ => None,
        }
    }
}

impl<'s, 'f> Slot<'s, 'f> {
    /// Fill the slot with an error.
    ///
    /// The given value doesn't need to satisfy any particular lifetime constraints.
    ///
    /// # Panics
    ///
    /// Calling more than a single `fill` method on this slot will panic.
    pub fn fill_error<T>(&mut self, value: T) -> Result<(), crate::Error>
    where
        T: error::Error + 'static,
    {
        self.fill(|visitor| visitor.error(&value))
    }

    /// Fill the slot with an error.
    pub fn fill_dyn_error(&mut self, value: &(dyn error::Error + 'static)) -> Result<(), crate::Error> {
        self.fill(|visitor| visitor.error(value))
    }
}

pub use self::error::Error;

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test_configure!(run_in_browser);

    use super::*;

    use crate::{
        std::{io, string::ToString},
        test::*,
    };

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn error_capture() {
        let err = io::Error::from(io::ErrorKind::Other);

        assert_eq!(
            err.to_string(),
            ValueBag::capture_error(&err)
                .to_borrowed_error()
                .expect("invalid value")
                .to_string()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn error_downcast() {
        let err = io::Error::from(io::ErrorKind::Other);

        assert!(ValueBag::capture_error(&err)
            .downcast_ref::<io::Error>()
            .is_some());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn error_visit() {
        let err = io::Error::from(io::ErrorKind::Other);

        ValueBag::from_dyn_error(&err).visit(TestVisit).expect("failed to visit value");
    }
}
