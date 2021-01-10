//! Structured values.

#![cfg_attr(value_bag_capture_const_type_id, feature(const_type_id))]
#![doc(html_root_url = "https://docs.rs/value-bag/1.0.0-alpha.5")]
#![no_std]

#[cfg(any(feature = "std", test))]
#[macro_use]
#[allow(unused_imports)]
extern crate std;

#[cfg(not(any(feature = "std", test)))]
#[macro_use]
#[allow(unused_imports)]
extern crate core as std;

mod error;
pub mod fill;
pub mod visit;
mod impls;
mod internal;

#[cfg(any(test, feature = "test"))]
pub mod test;

pub use self::error::Error;

/// A dynamic structured value.
///
/// # Capturing values
///
/// There are a few ways to capture a value:
///
/// - Using the `ValueBag::capture_*` methods.
/// - Using the standard `From` trait.
/// - Using the `Fill` API.
///
/// ## Using the `ValueBag::capture_*` methods
///
/// `ValueBag` offers a few constructor methods that capture values of different kinds.
/// These methods require a `T: 'static` to support downcasting.
///
/// ```
/// use value_bag::ValueBag;
///
/// let value = ValueBag::capture_debug(&42i32);
///
/// assert_eq!(Some(42), value.to_i32());
/// ```
///
/// Capturing a value using these methods will retain type information so that
/// the contents of the bag can be serialized using an appropriate type.
///
/// For cases where the `'static` bound can't be satisfied, there's also a few
/// constructors that exclude it.
///
/// ```
/// # use std::fmt::Debug;
/// use value_bag::ValueBag;
///
/// let value = ValueBag::from_debug(&42i32);
///
/// assert_eq!(None, value.to_i32());
/// ```
///
/// These `ValueBag::from_*` methods are lossy though and `ValueBag::capture_*` should be preferred.
///
/// ## Using the standard `From` trait
///
/// Primitive types can be converted into a `ValueBag` using the standard `From` trait.
///
/// ```
/// use value_bag::ValueBag;
///
/// let value = ValueBag::from(42i32);
///
/// assert_eq!(Some(42), value.to_i32());
/// ```
///
/// ## Using the `Fill` API
///
/// The `Fill` trait is a way to bridge APIs that may not be directly
/// compatible with other constructor methods.
///
/// The `Fill` trait is automatically implemented for `Fn`, so can usually
/// be used in libraries that can't implement the trait themselves:
///
/// ```
/// use value_bag::{ValueBag, fill::Slot};
///
/// let value = ValueBag::from_fill(&|slot: &mut Slot| {
///     #[derive(Debug)]
///     struct MyShortLivedValue;
///
///     slot.fill_debug(&MyShortLivedValue)
/// });
///
/// assert_eq!("MyShortLivedValue", format!("{:?}", value));
/// ```
///
/// The trait can also be implemented manually:
///
/// ```
/// # use std::fmt::Debug;
/// use value_bag::{ValueBag, Error, fill::{Slot, Fill}};
///
/// struct FillDebug;
///
/// impl Fill for FillDebug {
///     fn fill(&self, slot: &mut Slot) -> Result<(), Error> {
///         slot.fill_debug(&42i32 as &dyn Debug)
///     }
/// }
///
/// let value = ValueBag::from_fill(&FillDebug);
///
/// assert_eq!(None, value.to_i32());
/// ```
///
/// # Inspecting values
///
/// Once you have a `ValueBag` there are also a few ways to inspect it:
///
/// - Using the `Debug`, `Display`, `Serialize`, and `Stream` trait implementations.
/// - Using the `ValueBag::visit` method.
/// - Using the `ValueBag::to_*` methods.
/// - Using the `ValueBag::downcast_ref` method.
///
/// ## Using the trait implementations
///
/// ## Using the `ValueBag::visit` method
///
/// ## Using the `ValueBag::to_*` methods
///
/// ## Using the `ValueBag::downcast_ref` method
#[derive(Clone)]
pub struct ValueBag<'v> {
    inner: internal::Internal<'v>,
}
