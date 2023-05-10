#![cfg(feature = "alloc")]
#![feature(test)]

extern crate test;

use value_bag::ValueBag;

#[bench]
fn u8_to_owned(b: &mut test::Bencher) {
    let bag = ValueBag::from(1u8);

    b.iter(|| bag.to_owned());
}

#[bench]
fn str_to_owned(b: &mut test::Bencher) {
    let bag = ValueBag::from("a string");

    b.iter(|| bag.to_owned());
}

#[bench]
fn display_to_owned(b: &mut test::Bencher) {
    let bag = ValueBag::from_display(&42);

    b.iter(|| bag.to_owned());
}
