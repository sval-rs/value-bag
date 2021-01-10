//! Primitive value visitor.

use crate::{
    ValueBag, Error,
    internal::{self, InternalVisitor},
};

/// A visitor for a `ValueBag`.
pub trait Visit<'v> {
    /// Visit a `ValueBag`.
    ///
    /// This is the only required method on `Visit` and acts as a fallback for any
    /// more specific methods that aren't overridden.
    /// The `ValueBag` may be formatted using its fmt::Debug` or `fmt::Display` implementation,
    /// or serialized using its `sval::Value` or `serde::Serialize` implementation.
    fn visit_any(&mut self, value: ValueBag) -> Result<(), Error>;

    /// Visit an unsigned integer.
    #[cfg(not(test))]
    fn visit_u64(&mut self, value: u64) -> Result<(), Error> {
        self.visit_any(value.into())
    }
    #[cfg(test)]
    fn visit_u64(&mut self, value: u64) -> Result<(), Error>;

    /// Visit a signed integer.
    #[cfg(not(test))]
    fn visit_i64(&mut self, value: i64) -> Result<(), Error> {
        self.visit_any(value.into())
    }
    #[cfg(test)]
    fn visit_i64(&mut self, value: i64) -> Result<(), Error>;

    /// Visit a floating point.
    #[cfg(not(test))]
    fn visit_f64(&mut self, value: f64) -> Result<(), Error> {
        self.visit_any(value.into())
    }
    #[cfg(test)]
    fn visit_f64(&mut self, value: f64) -> Result<(), Error>;

    /// Visit a boolean.
    #[cfg(not(test))]
    fn visit_bool(&mut self, value: bool) -> Result<(), Error> {
        self.visit_any(value.into())
    }
    #[cfg(test)]
    fn visit_bool(&mut self, value: bool) -> Result<(), Error>;

    /// Visit a string.
    #[cfg(not(test))]
    fn visit_str(&mut self, value: &str) -> Result<(), Error> {
        self.visit_any(value.into())
    }
    #[cfg(test)]
    fn visit_str(&mut self, value: &str) -> Result<(), Error>;

    /// Visit a string.
    #[cfg(not(test))]
    fn visit_borrowed_str(&mut self, value: &'v str) -> Result<(), Error> {
        self.visit_str(value)
    }
    #[cfg(test)]
    fn visit_borrowed_str(&mut self, value: &'v str) -> Result<(), Error>;

    /// Visit a Unicode character.
    #[cfg(not(test))]
    fn visit_char(&mut self, value: char) -> Result<(), Error> {
        let mut b = [0; 4];
        self.visit_str(&*value.encode_utf8(&mut b))
    }
    #[cfg(test)]
    fn visit_char(&mut self, value: char) -> Result<(), Error>;

    /// Visit an error.
    #[cfg(not(test))]
    #[cfg(feature = "error")]
    fn visit_error(&mut self, err: &(dyn crate::std::error::Error + 'static)) -> Result<(), Error> {
        self.visit_any(ValueBag::from_dyn_error(err))
    }
    #[cfg(test)]
    #[cfg(feature = "error")]
    fn visit_error(&mut self, err: &(dyn crate::std::error::Error + 'static)) -> Result<(), Error>;

    /// Visit an error.
    #[cfg(not(test))]
    #[cfg(feature = "error")]
    fn visit_borrowed_error(&mut self, err: &'v (dyn crate::std::error::Error + 'static)) -> Result<(), Error> {
        self.visit_any(ValueBag::from_dyn_error(err))
    }
    #[cfg(test)]
    #[cfg(feature = "error")]
    fn visit_borrowed_error(&mut self, err: &'v (dyn crate::std::error::Error + 'static)) -> Result<(), Error>;
}

impl<'v> ValueBag<'v> {
    pub fn visit(&self, visitor: impl Visit<'v>) -> Result<(), Error> {
        struct Visitor<V>(V);

        impl<'v, V> InternalVisitor<'v> for Visitor<V> where V: Visit<'v> {
            fn debug(&mut self, v: &dyn internal::fmt::Debug) -> Result<(), Error> {
                self.0.visit_any(ValueBag::from_dyn_debug(v))
            }

            fn display(&mut self, v: &dyn internal::fmt::Display) -> Result<(), Error> {
                self.0.visit_any(ValueBag::from_dyn_display(v))
            }

            fn u64(&mut self, v: u64) -> Result<(), Error> {
                self.0.visit_u64(v)
            }

            fn i64(&mut self, v: i64) -> Result<(), Error> {
                self.0.visit_i64(v)
            }

            fn f64(&mut self, v: f64) -> Result<(), Error> {
                self.0.visit_f64(v)
            }

            fn bool(&mut self, v: bool) -> Result<(), Error> {
                self.0.visit_bool(v)
            }

            fn char(&mut self, v: char) -> Result<(), Error> {
                self.0.visit_char(v)
            }

            fn str(&mut self, v: &str) -> Result<(), Error> {
                self.0.visit_str(v)
            }

            fn borrowed_str(&mut self, v: &'v str) -> Result<(), Error> {
                self.0.visit_borrowed_str(v)
            }

            fn none(&mut self) -> Result<(), Error> {
                self.0.visit_any(ValueBag::from(()))
            }

            #[cfg(feature = "error")]
            fn error(&mut self, v: &(dyn internal::error::Error + 'static)) -> Result<(), Error> {
                self.0.visit_error(v)
            }

            #[cfg(feature = "error")]
            fn borrowed_error(&mut self, v: &'v (dyn internal::error::Error + 'static)) -> Result<(), Error> {
                self.0.visit_borrowed_error(v)
            }

            #[cfg(feature = "sval1")]
            fn sval1(&mut self, v: &dyn internal::sval::v1::Value) -> Result<(), Error> {
                internal::sval::v1::internal_visit(v, self)
            }

            #[cfg(feature = "serde1")]
            fn serde1(&mut self, v: &dyn internal::serde::v1::Serialize) -> Result<(), Error> {
                internal::serde::v1::internal_visit(v, self)
            }
        }

        self.internal_visit(&mut Visitor(visitor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visit_structured() {
        struct VisitStructured;

        impl<'v> Visit<'v> for VisitStructured {
            fn visit_any(&mut self, v: ValueBag) -> Result<(), Error> {
                panic!("unexpected value: {}", v)
            }

            fn visit_i64(&mut self, v: i64) -> Result<(), Error> {
                assert_eq!(-42i64, v);
                Ok(())
            }

            fn visit_u64(&mut self, v: u64) -> Result<(), Error> {
                assert_eq!(42u64, v);
                Ok(())
            }

            fn visit_f64(&mut self, v: f64) -> Result<(), Error> {
                assert_eq!(11f64, v);
                Ok(())
            }

            fn visit_bool(&mut self, v: bool) -> Result<(), Error> {
                assert_eq!(true, v);
                Ok(())
            }

            fn visit_str(&mut self, v: &str) -> Result<(), Error> {
                assert_eq!("some string", v);
                Ok(())
            }

            fn visit_borrowed_str(&mut self, v: &'v str) -> Result<(), Error> {
                assert_eq!("some string", v);
                Ok(())
            }

            fn visit_char(&mut self, v: char) -> Result<(), Error> {
                assert_eq!('n', v);
                Ok(())
            }

            fn visit_error(&mut self, err: &(dyn crate::std::error::Error + 'static)) -> Result<(), Error> {
                panic!("unexpected value: {}", err)
            }

            fn visit_borrowed_error(&mut self, err: &'v (dyn crate::std::error::Error + 'static)) -> Result<(), Error> {
                panic!("unexpected value: {}", err)
            }
        }

        unimplemented!()
    }

    #[cfg(all(feature = "sval", feature = "std"))]
    mod sval_error {
        use super::*;

        use crate::{
            std::{error, io, string::ToString},
            internal::sval::v1 as sval,
        };

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn sval1_error() {
            let err: &(dyn error::Error + 'static) = &io::Error::from(io::ErrorKind::Other);
            let value: &dyn sval::Value = &err;

            struct VisitError<'a>(&'a (dyn error::Error + 'static));

            impl<'v> Visit<'v> for VisitError<'v> {
                fn visit_any(&mut self, v: ValueBag) -> Result<(), Error> {
                    panic!("unexpected value: {}", v)
                }

                fn visit_i64(&mut self, v: i64) -> Result<(), Error> {
                    panic!("unexpected value: {}", v)
                }

                fn visit_u64(&mut self, v: u64) -> Result<(), Error> {
                    panic!("unexpected value: {}", v)
                }

                fn visit_f64(&mut self, v: f64) -> Result<(), Error> {
                    panic!("unexpected value: {}", v)
                }

                fn visit_bool(&mut self, v: bool) -> Result<(), Error> {
                    panic!("unexpected value: {}", v)
                }

                fn visit_str(&mut self, v: &str) -> Result<(), Error> {
                    panic!("unexpected value: {}", v)
                }

                fn visit_borrowed_str(&mut self, v: &'v str) -> Result<(), Error> {
                    panic!("unexpected value: {}", v)
                }

                fn visit_char(&mut self, v: char) -> Result<(), Error> {
                    panic!("unexpected value: {}", v)
                }

                fn visit_error(&mut self, err: &(dyn crate::std::error::Error + 'static)) -> Result<(), Error> {
                    assert_eq!(self.0.to_string(), err.to_string());
                    Ok(())
                }

                fn visit_borrowed_error(&mut self, err: &'v (dyn crate::std::error::Error + 'static)) -> Result<(), Error> {
                    self.visit_error(err)
                }
            }

            // Ensure that an error captured through `sval` can be visited as an error
            ValueBag::from_dyn_sval1(value).visit(VisitError(err)).expect("failed to visit value");
        }
    }
}
