//! Integration between `Value` and `sval`.
//!
//! This module allows any `Value` to implement the `Value` trait,
//! and for any `Value` to be captured as a `Value`.

use crate::{
    fill::Slot,
    internal::{Internal, InternalVisitor},
    std::{any::Any, fmt},
    Error, ValueBag,
};

impl<'v> ValueBag<'v> {
    /// Get a value from a structured type.
    ///
    /// This method will attempt to capture the given value as a well-known primitive
    /// before resorting to using its `Value` implementation.
    pub fn capture_sval2<T>(value: &'v T) -> Self
    where
        T: value_bag_sval2::lib::Value + 'static,
    {
        Self::try_capture(value).unwrap_or(ValueBag {
            inner: Internal::Sval2(value),
        })
    }

    /// Get a value from a structured type without capturing support.
    pub fn from_sval2<T>(value: &'v T) -> Self
    where
        T: value_bag_sval2::lib::Value,
    {
        ValueBag {
            inner: Internal::AnonSval2(value),
        }
    }

    /// Get a value from a structured type without capturing support.
    #[inline]
    pub fn from_dyn_sval2(value: &'v dyn Value) -> Self {
        ValueBag {
            inner: Internal::AnonSval2(value),
        }
    }
}

pub(crate) trait DowncastValue {
    fn as_any(&self) -> &dyn Any;
    fn as_super(&self) -> &dyn Value;
}

impl<T: value_bag_sval2::lib::Value + 'static> DowncastValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_super(&self) -> &dyn Value {
        self
    }
}

impl<'s, 'f> Slot<'s, 'f> {
    /// Fill the slot with a structured value.
    ///
    /// The given value doesn't need to satisfy any particular lifetime constraints.
    pub fn fill_sval2<T>(self, value: T) -> Result<(), Error>
    where
        T: value_bag_sval2::lib::Value,
    {
        self.fill(|visitor| visitor.sval2(&value))
    }
}

impl<'v> value_bag_sval2::lib::Value for ValueBag<'v> {
    fn stream<'sval, S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
        &'sval self,
        s: &mut S,
    ) -> value_bag_sval2::lib::Result {
        use value_bag_sval2::lib_ref::ValueRef as _;

        self.stream_ref(s)
    }
}

impl<'sval> value_bag_sval2::lib_ref::ValueRef<'sval> for ValueBag<'sval> {
    fn stream_ref<S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
        &self,
        s: &mut S,
    ) -> value_bag_sval2::lib::Result {
        struct Sval2Visitor<'a, S: ?Sized>(&'a mut S);

        impl<'a, 'v, S: value_bag_sval2::lib::Stream<'v> + ?Sized> InternalVisitor<'v>
            for Sval2Visitor<'a, S>
        {
            fn debug(&mut self, v: &dyn fmt::Debug) -> Result<(), Error> {
                value_bag_sval2::fmt::stream_debug(self.0, v).map_err(Error::from_sval2)
            }

            fn display(&mut self, v: &dyn fmt::Display) -> Result<(), Error> {
                value_bag_sval2::fmt::stream_display(self.0, v).map_err(Error::from_sval2)
            }

            fn seq_elem(&mut self, v: ValueBag) -> Result<(), Error> {
                self.0.value_computed(&v).map_err(Error::from_sval2)
            }

            fn borrowed_seq_elem(&mut self, v: ValueBag<'v>) -> Result<(), Error> {
                use value_bag_sval2::lib_ref::ValueRef as _;

                v.stream_ref(self.0).map_err(Error::from_sval2)
            }

            fn u64(&mut self, v: u64) -> Result<(), Error> {
                self.0.u64(v).map_err(Error::from_sval2)
            }

            fn i64(&mut self, v: i64) -> Result<(), Error> {
                self.0.i64(v).map_err(Error::from_sval2)
            }

            fn u128(&mut self, v: &u128) -> Result<(), Error> {
                self.0.u128(*v).map_err(Error::from_sval2)
            }

            fn i128(&mut self, v: &i128) -> Result<(), Error> {
                self.0.i128(*v).map_err(Error::from_sval2)
            }

            fn f64(&mut self, v: f64) -> Result<(), Error> {
                self.0.f64(v).map_err(Error::from_sval2)
            }

            fn bool(&mut self, v: bool) -> Result<(), Error> {
                self.0.bool(v).map_err(Error::from_sval2)
            }

            fn char(&mut self, v: char) -> Result<(), Error> {
                let mut buf = [0; 4];
                let v = v.encode_utf8(&mut buf);

                self.0.value_computed(v).map_err(Error::from_sval2)
            }

            fn str(&mut self, v: &str) -> Result<(), Error> {
                self.0.value_computed(v).map_err(Error::from_sval2)
            }

            fn borrowed_str(&mut self, v: &'v str) -> Result<(), Error> {
                self.0.value(v).map_err(Error::from_sval2)
            }

            fn none(&mut self) -> Result<(), Error> {
                self.0.null().map_err(Error::from_sval2)
            }

            #[cfg(feature = "error")]
            fn error(&mut self, v: &(dyn std::error::Error + 'static)) -> Result<(), Error> {
                self.display(&v)
            }

            fn sval2(&mut self, v: &dyn Value) -> Result<(), Error> {
                self.0.value_computed(v).map_err(Error::from_sval2)
            }

            fn borrowed_sval2(&mut self, v: &'v dyn Value) -> Result<(), Error> {
                self.0.value(v).map_err(Error::from_sval2)
            }

            #[cfg(feature = "serde1")]
            fn serde1(
                &mut self,
                v: &dyn crate::internal::serde::v1::Serialize,
            ) -> Result<(), Error> {
                crate::internal::serde::v1::sval2(self.0, v)
            }

            fn poisoned(&mut self, msg: &'static str) -> Result<(), Error> {
                Err(Error::msg(msg))
            }
        }

        self.internal_visit(&mut Sval2Visitor(s))
            .map_err(Error::into_sval2)?;

        Ok(())
    }
}

pub use value_bag_sval2::dynamic::Value;

pub(in crate::internal) fn fmt(f: &mut fmt::Formatter, v: &dyn Value) -> Result<(), Error> {
    value_bag_sval2::fmt::stream_to_fmt(f, v)?;
    Ok(())
}

#[cfg(feature = "serde1")]
pub(in crate::internal) fn serde1<S>(s: S, v: &dyn Value) -> Result<S::Ok, S::Error>
where
    S: value_bag_serde1::lib::Serializer,
{
    value_bag_sval2::serde1::serialize(s, v)
}

pub(crate) fn internal_visit<'v>(
    v: &dyn Value,
    visitor: &mut dyn InternalVisitor<'v>,
) -> Result<(), Error> {
    let mut visitor = VisitorStream {
        internal: VisitorInternal {
            visitor,
            depth: 0,
            position: Position::Root,
        },
        text_buf: Default::default(),
    };

    value_bag_sval2::lib::stream_computed(&mut visitor, v).map_err(Error::from_sval2)?;

    Ok(())
}

pub(crate) fn borrowed_internal_visit<'v>(
    v: &'v dyn Value,
    visitor: &mut dyn InternalVisitor<'v>,
) -> Result<(), Error> {
    let mut visitor = VisitorStream {
        internal: VisitorInternal {
            visitor,
            depth: 0,
            position: Position::Root,
        },
        text_buf: Default::default(),
    };

    value_bag_sval2::lib::stream(&mut visitor, v).map_err(Error::from_sval2)?;

    Ok(())
}

enum Position {
    Root,
    MapKey,
    MapValue,
    SeqElem,
}

struct VisitorStream<'a, 'v> {
    internal: VisitorInternal<'a, 'v>,
    text_buf: value_bag_sval2::buffer::TextBuf<'v>,
}

struct VisitorInternal<'a, 'v> {
    visitor: &'a mut dyn InternalVisitor<'v>,
    position: Position,
    depth: usize,
}

impl<'a, 'v> VisitorInternal<'a, 'v> {
    fn visit<'b>(
        &mut self,
        root: impl FnOnce(&mut dyn InternalVisitor<'v>) -> Result<(), Error>,
        value: impl Into<ValueBag<'b>>,
    ) -> value_bag_sval2::lib::Result {
        if self.depth > 1 {
            return Ok(());
        }

        match self.position {
            Position::Root => root(self.visitor),
            Position::SeqElem => self.visitor.seq_elem(value.into()),
            Position::MapKey => Err(Error::msg("maps are not supported")),
            Position::MapValue => Err(Error::msg("maps are not supported")),
        }
        .map_err(Error::into_sval2)
    }

    fn borrowed_visit(
        &mut self,
        root: impl FnOnce(&mut dyn InternalVisitor<'v>) -> Result<(), Error>,
        value: impl Into<ValueBag<'v>>,
    ) -> value_bag_sval2::lib::Result {
        if self.depth > 1 {
            return Ok(());
        }

        match self.position {
            Position::Root => root(self.visitor),
            Position::SeqElem => self.visitor.borrowed_seq_elem(value.into()),
            Position::MapKey => Err(Error::msg("maps are not supported")),
            Position::MapValue => Err(Error::msg("maps are not supported")),
        }
        .map_err(Error::into_sval2)
    }
}

impl<'a, 'v> value_bag_sval2::lib::Stream<'v> for VisitorStream<'a, 'v> {
    fn null(&mut self) -> value_bag_sval2::lib::Result {
        self.internal.visit(|visitor| visitor.none(), ())
    }

    fn bool(&mut self, v: bool) -> value_bag_sval2::lib::Result {
        self.internal.visit(|visitor| visitor.bool(v), v)
    }

    fn i64(&mut self, v: i64) -> value_bag_sval2::lib::Result {
        self.internal.visit(|visitor| visitor.i64(v), v)
    }

    fn u64(&mut self, v: u64) -> value_bag_sval2::lib::Result {
        self.internal.visit(|visitor| visitor.u64(v), v)
    }

    fn i128(&mut self, v: i128) -> value_bag_sval2::lib::Result {
        self.internal.visit(|visitor| visitor.i128(&v), &v)
    }

    fn u128(&mut self, v: u128) -> value_bag_sval2::lib::Result {
        self.internal.visit(|visitor| visitor.u128(&v), &v)
    }

    fn f64(&mut self, v: f64) -> value_bag_sval2::lib::Result {
        self.internal.visit(|visitor| visitor.f64(v), v)
    }

    fn text_begin(&mut self, _: Option<usize>) -> value_bag_sval2::lib::Result {
        if self.internal.depth > 1 {
            return Ok(());
        }

        self.text_buf.clear();
        Ok(())
    }

    fn text_fragment_computed(&mut self, f: &str) -> value_bag_sval2::lib::Result {
        if self.internal.depth > 1 {
            return Ok(());
        }

        self.text_buf
            .push_fragment_computed(f)
            .map_err(|_| value_bag_sval2::lib::Error::new())
    }

    fn text_fragment(&mut self, f: &'v str) -> value_bag_sval2::lib::Result {
        if self.internal.depth > 1 {
            return Ok(());
        }

        self.text_buf
            .push_fragment(f)
            .map_err(|_| value_bag_sval2::lib::Error::new())
    }

    fn text_end(&mut self) -> value_bag_sval2::lib::Result {
        if let Some(v) = self.text_buf.as_borrowed_str() {
            self.internal
                .borrowed_visit(|visitor| visitor.borrowed_str(v), v)
        } else {
            self.internal.visit(
                |visitor| visitor.str(self.text_buf.as_str()),
                self.text_buf.as_str(),
            )
        }
    }

    fn seq_begin(&mut self, _: Option<usize>) -> value_bag_sval2::lib::Result {
        self.internal.depth += 1;

        if self.internal.depth != 1 {
            self.internal.visit(|visitor| visitor.none(), ())?;
        }

        Ok(())
    }

    fn seq_end(&mut self) -> value_bag_sval2::lib::Result {
        self.internal.depth -= 1;
        Ok(())
    }

    fn map_begin(&mut self, _: Option<usize>) -> value_bag_sval2::lib::Result {
        self.internal.depth += 1;
        self.internal.visit(|visitor| visitor.none(), ())
    }

    fn map_end(&mut self) -> value_bag_sval2::lib::Result {
        self.internal.depth -= 1;
        Ok(())
    }

    fn seq_value_begin(&mut self) -> value_bag_sval2::lib::Result {
        self.internal.position = Position::SeqElem;
        Ok(())
    }

    fn seq_value_end(&mut self) -> value_bag_sval2::lib::Result {
        Ok(())
    }

    fn map_key_begin(&mut self) -> value_bag_sval2::lib::Result {
        self.internal.position = Position::MapKey;
        Ok(())
    }

    fn map_key_end(&mut self) -> value_bag_sval2::lib::Result {
        Ok(())
    }

    fn map_value_begin(&mut self) -> value_bag_sval2::lib::Result {
        self.internal.position = Position::MapValue;
        Ok(())
    }

    fn map_value_end(&mut self) -> value_bag_sval2::lib::Result {
        Ok(())
    }
}

impl Error {
    pub(in crate::internal) fn from_sval2(_: value_bag_sval2::lib::Error) -> Self {
        Error::msg("`sval` serialization failed")
    }

    pub(in crate::internal) fn into_sval2(self) -> value_bag_sval2::lib::Error {
        value_bag_sval2::lib::Error::new()
    }
}

#[cfg(feature = "owned")]
pub(crate) mod owned {
    impl value_bag_sval2::lib::Value for crate::OwnedValueBag {
        fn stream<'sval, S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
            &'sval self,
            s: &mut S,
        ) -> value_bag_sval2::lib::Result {
            value_bag_sval2::lib_ref::ValueRef::stream_ref(&self.by_ref(), s)
        }
    }

    pub(crate) type OwnedValue = value_bag_sval2::buffer::Value<'static>;

    pub(crate) fn buffer(
        v: impl value_bag_sval2::lib::Value,
    ) -> Result<OwnedValue, value_bag_sval2::buffer::Error> {
        OwnedValue::collect_owned(v)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    use super::*;
    use crate::test::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_capture() {
        assert_eq!(
            ValueBag::capture_sval2(&42u64).to_test_token(),
            TestToken::U64(42)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_capture_cast() {
        assert_eq!(
            42u64,
            ValueBag::capture_sval2(&42u64)
                .to_u64()
                .expect("invalid value")
        );

        assert_eq!(
            "a string",
            ValueBag::capture_sval2(&"a string")
                .to_borrowed_str()
                .expect("invalid value")
        );

        #[cfg(feature = "alloc")]
        assert_eq!(
            "a string",
            ValueBag::capture_sval2(&"a string")
                .to_str()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_capture_cast_borrowed_str() {
        struct Number<'a>(&'a str);

        impl<'a> value_bag_sval2::lib::Value for Number<'a> {
            fn stream<'sval, S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
                &'sval self,
                stream: &mut S,
            ) -> value_bag_sval2::lib::Result {
                stream.tagged_begin(Some(&value_bag_sval2::lib::tags::NUMBER), None, None)?;
                stream.value(self.0)?;
                stream.tagged_end(Some(&value_bag_sval2::lib::tags::NUMBER), None, None)
            }
        }

        assert_eq!(
            "123.456e789",
            ValueBag::capture_sval2(&Number("123.456e789"))
                .to_borrowed_str()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_from_cast() {
        assert_eq!(
            42u64,
            ValueBag::from_sval2(&42u64)
                .to_u64()
                .expect("invalid value")
        );

        #[cfg(feature = "alloc")]
        assert_eq!(
            "a string",
            ValueBag::from_sval2(&"a string")
                .to_str()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_downcast() {
        #[derive(Debug, PartialEq, Eq)]
        struct Timestamp(usize);

        impl value_bag_sval2::lib::Value for Timestamp {
            fn stream<'sval, S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
                &'sval self,
                stream: &mut S,
            ) -> value_bag_sval2::lib::Result {
                stream.u64(self.0 as u64)
            }
        }

        let ts = Timestamp(42);

        assert_eq!(
            &ts,
            ValueBag::capture_sval2(&ts)
                .downcast_ref::<Timestamp>()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_structured() {
        let value = ValueBag::from(42u64);

        value_bag_sval2::test::assert_tokens(&value, &[value_bag_sval2::test::Token::U64(42)]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_debug() {
        struct TestSval;

        impl value_bag_sval2::lib::Value for TestSval {
            fn stream<'sval, S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
                &'sval self,
                stream: &mut S,
            ) -> value_bag_sval2::lib::Result {
                stream.u64(42)
            }
        }

        assert_eq!(
            format!("{:04?}", 42u64),
            format!("{:04?}", ValueBag::capture_sval2(&TestSval)),
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_visit() {
        ValueBag::from_sval2(&42u64)
            .visit(TestVisit::default())
            .expect("failed to visit value");
        ValueBag::from_sval2(&-42i64)
            .visit(TestVisit::default())
            .expect("failed to visit value");
        ValueBag::from_sval2(&11f64)
            .visit(TestVisit::default())
            .expect("failed to visit value");
        ValueBag::from_sval2(&true)
            .visit(TestVisit::default())
            .expect("failed to visit value");
        ValueBag::from_sval2(&"some borrowed string")
            .visit(TestVisit::default())
            .expect("failed to visit value");
        ValueBag::from_sval2(&'n')
            .visit(TestVisit {
                str: "n",
                ..Default::default()
            })
            .expect("failed to visit value");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg(feature = "serde1")]
    fn sval2_serde1() {
        use value_bag_serde1::test::{assert_ser_tokens, Token};

        struct TestSval;

        impl value_bag_sval2::lib::Value for TestSval {
            fn stream<'sval, S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
                &'sval self,
                stream: &mut S,
            ) -> value_bag_sval2::lib::Result {
                stream.u64(42)
            }
        }

        assert_ser_tokens(&ValueBag::capture_sval2(&TestSval), &[Token::U64(42)]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_collect() {
        use value_bag_sval2::dynamic::Value;

        use crate::std::vec::Vec;

        let value = ValueBag::from_sval2(&[
            &1 as &dyn Value,
            &"string" as &dyn Value,
            &2 as &dyn Value,
            &3 as &dyn Value,
        ]);

        let mut vec = Vec::<Option<f64>>::new();
        value.collect_f64(&mut vec);
        assert_eq!(vec, vec![Some(1.0), None, Some(2.0), Some(3.0)]);

        let mut vec = Vec::<Option<&str>>::new();
        value.collect_borrowed_str(&mut vec);
        assert_eq!(vec, vec![None, Some("string"), None, None]);
    }

    #[cfg(feature = "alloc")]
    mod alloc_support {
        use super::*;

        use crate::std::borrow::ToOwned;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn sval2_cast() {
            assert_eq!(
                "a string",
                ValueBag::capture_sval2(&"a string".to_owned())
                    .by_ref()
                    .to_str()
                    .expect("invalid value")
            );
        }
    }

    #[cfg(feature = "owned")]
    mod owned_support {
        use super::*;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn sval2_to_owned_poison() {
            struct Kaboom;

            impl value_bag_sval2::lib::Value for Kaboom {
                fn stream<'sval, S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
                    &'sval self,
                    _: &mut S,
                ) -> value_bag_sval2::lib::Result {
                    value_bag_sval2::lib::error()
                }
            }

            let value = ValueBag::capture_sval2(&Kaboom)
                .to_owned()
                .by_ref()
                .to_test_token();

            assert_eq!(
                TestToken::Poisoned("failed to buffer the value".into()),
                value
            );
        }
    }
}
