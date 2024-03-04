use crate::std::{ops::ControlFlow, fmt, marker::PhantomData};

use crate::{
    internal::{Internal, InternalVisitor},
    Error, ValueBag,
    fill::{Fill, Slot},
    visit::Visit,
};

pub fn for_each_continue() -> ControlFlow<()> {
    ControlFlow::Continue(())
}

pub fn for_each_break() -> ControlFlow<()> {
    ControlFlow::Break(())
}

pub(crate) fn visit<'a, 'v>(
    v: &dyn ForEachValue<'a>,
    visitor: &mut dyn Visit<'v>,
) -> Result<(), Error> {
    visitor.visit_any(ValueBag::from_fill(&|slot: crate::fill::Slot| slot.fill(|visitor| visitor.seq(v))))
}

#[derive(Default)]
pub(crate) struct ExtendPrimitive<S, T>(S, PhantomData<T>);

impl<S, T> ExtendPrimitive<S, T> {
    pub fn into_inner(self) -> S {
        self.0
    }
}

impl<'a, S: Extend<Option<T>>, T: for<'b> TryFrom<ValueBag<'b>>> ExtendValue<'a>
    for ExtendPrimitive<S, T>
{
    fn extend<'b>(&mut self, inner: Internal<'b>) {
        self.0.extend(Some(ValueBag { inner }.try_into().ok()))
    }
}

#[allow(dead_code)]
pub(crate) trait ExtendValue<'v> {
    fn extend<'a>(&mut self, v: Internal<'a>);

    fn extend_borrowed(&mut self, v: Internal<'v>) {
        self.extend(v);
    }
}

pub(crate) trait ForEachValue<'v> {
    fn for_each(&self, f: &mut dyn FnMut(Internal<'v>) -> ControlFlow<()>);
}

impl<'v, F, I> ForEachValue<'v> for F
where
    F: Fn() -> I,
    I: Iterator<Item = Internal<'v>>,
{
    fn for_each(&self, f: &mut dyn FnMut(Internal<'v>) -> ControlFlow<()>) {
        for item in (self)() {
            if let ControlFlow::Break(()) = f(item) {
                return;
            }
        }
    }
}

impl<'v> Internal<'v> {
    #[inline]
    pub(crate) fn extend<S: Default + ExtendValue<'v>>(&self) -> Option<S> {
        struct SeqVisitor<S>(Option<S>);

        impl<'v, S: Default + ExtendValue<'v>> InternalVisitor<'v> for SeqVisitor<S> {
            #[inline]
            fn fill(&mut self, v: &dyn crate::fill::Fill) -> Result<(), Error> {
                v.fill(crate::fill::Slot::new(self))
            }

            #[inline]
            fn debug(&mut self, _: &dyn fmt::Debug) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn display(&mut self, _: &dyn fmt::Display) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn u64(&mut self, _: u64) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn i64(&mut self, _: i64) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn u128(&mut self, _: &u128) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn i128(&mut self, _: &i128) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn f64(&mut self, _: f64) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn bool(&mut self, _: bool) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn char(&mut self, _: char) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn str(&mut self, _: &str) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn none(&mut self) -> Result<(), Error> {
                Ok(())
            }

            #[cfg(feature = "error")]
            #[inline]
            fn error(&mut self, _: &dyn crate::internal::error::Error) -> Result<(), Error> {
                Ok(())
            }

            #[cfg(feature = "sval2")]
            #[inline]
            fn sval2(&mut self, v: &dyn crate::internal::sval::v2::Value) -> Result<(), Error> {
                self.0 = crate::internal::sval::v2::seq::extend(v);

                Ok(())
            }

            #[cfg(feature = "sval2")]
            #[inline]
            fn borrowed_sval2(
                &mut self,
                v: &'v dyn crate::internal::sval::v2::Value,
            ) -> Result<(), Error> {
                self.0 = crate::internal::sval::v2::seq::extend_borrowed(v);

                Ok(())
            }

            #[cfg(feature = "serde1")]
            #[inline]
            fn serde1(
                &mut self,
                v: &dyn crate::internal::serde::v1::Serialize,
            ) -> Result<(), Error> {
                self.0 = crate::internal::serde::v1::seq::extend(v);

                Ok(())
            }

            fn seq<'b>(&mut self, seq: &dyn ForEachValue<'b>) -> Result<(), Error> {
                let mut s = S::default();

                seq.for_each(&mut |v| {
                    s.extend(v);
                    for_each_continue()
                });

                self.0 = Some(s);

                Ok(())
            }

            fn borrowed_seq(&mut self, seq: &dyn ForEachValue<'v>) -> Result<(), Error> {
                let mut s = S::default();

                seq.for_each(&mut |v| {
                    s.extend_borrowed(v);
                    for_each_continue()
                });

                self.0 = Some(s);

                Ok(())
            }

            fn poisoned(&mut self, _: &'static str) -> Result<(), Error> {
                Ok(())
            }
        }

        let mut visitor = SeqVisitor(None);
        let _ = self.internal_visit(&mut visitor);

        visitor.0
    }
}
