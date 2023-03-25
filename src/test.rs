//! Test support for inspecting values.

use crate::{
    internal,
    std::{fmt, str, string::String},
    visit::Visit,
    Error, ValueBag,
};

pub(crate) trait IntoValueBag<'v> {
    fn into_value_bag(self) -> ValueBag<'v>;
}

impl<'v, T> IntoValueBag<'v> for T
where
    T: Into<ValueBag<'v>>,
{
    fn into_value_bag(self) -> ValueBag<'v> {
        self.into()
    }
}

/**
A tokenized representation of the captured value.
*/
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Token {
    U64(u64),
    I64(i64),
    F64(f64),
    U128(u128),
    I128(i128),
    Char(char),
    Bool(bool),
    Str(String),
    None,

    #[cfg(feature = "error")]
    Error,

    #[cfg(feature = "sval2")]
    Sval(Sval),

    #[cfg(feature = "serde1")]
    Serde(Serde),
}

/**
A value that was captured using `sval`.
*/
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct Sval {
    pub version: u32,
}

/**
A value that was captured using `serde`.
*/
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct Serde {
    pub version: u32,
}

impl<'v> ValueBag<'v> {
    /**
    Convert the value bag into a token for testing.

    This _isn't_ a general-purpose API for working with values outside of testing.
    */
    pub fn to_token(&self) -> Token {
        struct TestVisitor(Option<Token>);

        impl<'v> internal::InternalVisitor<'v> for TestVisitor {
            fn debug(&mut self, v: &dyn fmt::Debug) -> Result<(), Error> {
                self.0 = Some(Token::Str(format!("{:?}", v)));
                Ok(())
            }

            fn display(&mut self, v: &dyn fmt::Display) -> Result<(), Error> {
                self.0 = Some(Token::Str(format!("{}", v)));
                Ok(())
            }

            fn u64(&mut self, v: u64) -> Result<(), Error> {
                self.0 = Some(Token::U64(v));
                Ok(())
            }

            fn i64(&mut self, v: i64) -> Result<(), Error> {
                self.0 = Some(Token::I64(v));
                Ok(())
            }

            fn u128(&mut self, v: &u128) -> Result<(), Error> {
                self.0 = Some(Token::U128(*v));
                Ok(())
            }

            fn i128(&mut self, v: &i128) -> Result<(), Error> {
                self.0 = Some(Token::I128(*v));
                Ok(())
            }

            fn f64(&mut self, v: f64) -> Result<(), Error> {
                self.0 = Some(Token::F64(v));
                Ok(())
            }

            fn bool(&mut self, v: bool) -> Result<(), Error> {
                self.0 = Some(Token::Bool(v));
                Ok(())
            }

            fn char(&mut self, v: char) -> Result<(), Error> {
                self.0 = Some(Token::Char(v));
                Ok(())
            }

            fn str(&mut self, v: &str) -> Result<(), Error> {
                self.0 = Some(Token::Str(v.into()));
                Ok(())
            }

            fn none(&mut self) -> Result<(), Error> {
                self.0 = Some(Token::None);
                Ok(())
            }

            #[cfg(feature = "error")]
            fn error(&mut self, _: &dyn internal::error::Error) -> Result<(), Error> {
                self.0 = Some(Token::Error);
                Ok(())
            }

            #[cfg(feature = "sval2")]
            fn sval2(&mut self, _: &dyn internal::sval::v2::Value) -> Result<(), Error> {
                self.0 = Some(Token::Sval(Sval { version: 2 }));
                Ok(())
            }

            #[cfg(feature = "serde1")]
            fn serde1(&mut self, _: &dyn internal::serde::v1::Serialize) -> Result<(), Error> {
                self.0 = Some(Token::Serde(Serde { version: 1 }));
                Ok(())
            }
        }

        let mut visitor = TestVisitor(None);
        self.internal_visit(&mut visitor).unwrap();

        visitor.0.unwrap()
    }
}

pub(crate) struct TestVisit {
    pub i64: i64,
    pub u64: u64,
    pub i128: i128,
    pub u128: u128,
    pub f64: f64,
    pub bool: bool,
    pub str: &'static str,
    pub borrowed_str: &'static str,
    pub char: char,
}

impl Default for TestVisit {
    fn default() -> Self {
        TestVisit {
            i64: -42,
            u64: 42,
            i128: -42,
            u128: 42,
            f64: 11.0,
            bool: true,
            str: "some string",
            borrowed_str: "some borrowed string",
            char: 'n',
        }
    }
}

impl<'v> Visit<'v> for TestVisit {
    fn visit_any(&mut self, v: ValueBag) -> Result<(), Error> {
        panic!("unexpected value: {}", v)
    }

    fn visit_i64(&mut self, v: i64) -> Result<(), Error> {
        assert_eq!(self.i64, v);
        Ok(())
    }

    fn visit_u64(&mut self, v: u64) -> Result<(), Error> {
        assert_eq!(self.u64, v);
        Ok(())
    }

    fn visit_i128(&mut self, v: i128) -> Result<(), Error> {
        assert_eq!(self.i128, v);
        Ok(())
    }

    fn visit_u128(&mut self, v: u128) -> Result<(), Error> {
        assert_eq!(self.u128, v);
        Ok(())
    }

    fn visit_f64(&mut self, v: f64) -> Result<(), Error> {
        assert_eq!(self.f64, v);
        Ok(())
    }

    fn visit_bool(&mut self, v: bool) -> Result<(), Error> {
        assert_eq!(self.bool, v);
        Ok(())
    }

    fn visit_str(&mut self, v: &str) -> Result<(), Error> {
        assert_eq!(self.str, v);
        Ok(())
    }

    fn visit_borrowed_str(&mut self, v: &'v str) -> Result<(), Error> {
        assert_eq!(self.borrowed_str, v);
        Ok(())
    }

    fn visit_char(&mut self, v: char) -> Result<(), Error> {
        assert_eq!(self.char, v);
        Ok(())
    }

    #[cfg(feature = "error")]
    fn visit_error(&mut self, err: &(dyn crate::std::error::Error + 'static)) -> Result<(), Error> {
        assert!(err.downcast_ref::<crate::std::io::Error>().is_some());
        Ok(())
    }

    #[cfg(feature = "error")]
    fn visit_borrowed_error(
        &mut self,
        err: &'v (dyn crate::std::error::Error + 'static),
    ) -> Result<(), Error> {
        assert!(err.downcast_ref::<crate::std::io::Error>().is_some());
        Ok(())
    }
}
