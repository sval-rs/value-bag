# `value-bag`

![Rust](https://github.com/sval-rs/value-bag/workflows/Rust/badge.svg)

A `ValueBag` is an anonymous structured bag that supports casting, downcasting, formatting, and serializing. The goal of a `ValueBag` is to decouple the producers of structured data from its consumers. A `ValueBag` can _always_ be interrogated using the consumers serialization API of choice, even if that wasn't the one the producer used to capture the data in the first place.

Say we capture an `i32` using its `Display` implementation as a `ValueBag`:

```rust
let bag = ValueBag::capture_display(42);
```

That value can then be cast to a `u64`:

```rust
let num = bag.as_u64().unwrap();

assert_eq!(42, num);
```

It could also be serialized as a number using `serde`:

```rust
let num = serde_json::to_value(bag).unwrap();

assert!(num.is_number());
```

Say we derive `sval::Value` on a type and capture it as a `ValueBag`:

```rust
#[derive(Value)]
struct Work {
    id: u64,
    description: String,
}

let work = Work {
    id: 123,
    description: String::from("do the work"),
}

let bag = ValueBag::capture_sval(&work);
```

It could then be formatted using `Display`, even though `Work` never implemented that trait:

```rust
assert_eq!("Work { id: 123, description: \"do the work\" }", bag.to_string());
```

Or serialized using `serde` and retain its nested structure.

The tradeoff in all this is that `ValueBag` needs to depend on the serialization frameworks (`sval`, `serde`, and `std::fmt`) that it supports, instead of just providing an API of its own for others to plug into. Doing this lets `ValueBag` guarantee everything will always line up, and keep its own public API narrow. Each of these frameworks are stable though (except `sval` which is `1.0.0-alpha`).
