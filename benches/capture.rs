#![feature(test)]

extern crate test;

use value_bag::ValueBag;

#[bench]
fn u8_capture_from(b: &mut test::Bencher) {
    b.iter(|| ValueBag::from(1u8))
}

#[bench]
fn u8_capture_debug(b: &mut test::Bencher) {
    b.iter(|| ValueBag::capture_debug(&1u8))
}

#[bench]
fn str_capture_debug(b: &mut test::Bencher) {
    b.iter(|| ValueBag::capture_debug(&"a string"))
}

#[bench]
fn custom_capture_debug(b: &mut test::Bencher) {
    #[derive(Debug)]
    struct A;

    b.iter(|| ValueBag::capture_debug(&A))
}

#[bench]
fn u8_capture_from_to_u64(b: &mut test::Bencher) {
    let v = ValueBag::from(1u8);
    b.iter(|| v.to_u64())
}

#[bench]
fn u8_capture_debug_to_u64(b: &mut test::Bencher) {
    let v = ValueBag::capture_debug(&1u8);
    b.iter(|| v.to_u64())
}

#[bench]
fn u8_capture_debug_to_borrowed_str(b: &mut test::Bencher) {
    let v = ValueBag::capture_debug(&1u8);
    b.iter(|| v.to_borrowed_str())
}

#[bench]
fn str_capture_debug_to_borrowed_str(b: &mut test::Bencher) {
    let v = ValueBag::capture_debug(&"a string");
    b.iter(|| v.to_borrowed_str())
}

#[bench]
fn str_capture_debug_to_u64(b: &mut test::Bencher) {
    let v = ValueBag::capture_debug(&"a string");
    b.iter(|| v.to_u64())
}

#[bench]
fn custom_capture_debug_to_str(b: &mut test::Bencher) {
    #[derive(Debug)]
    struct A;

    let v = ValueBag::capture_debug(&A);
    b.iter(|| v.to_borrowed_str())
}
