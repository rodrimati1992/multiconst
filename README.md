[![Rust](https://github.com/rodrimati1992/multiconst/workflows/Rust/badge.svg)](https://github.com/rodrimati1992/multiconst/actions)
[![crates-io](https://img.shields.io/crates/v/multiconst.svg)](https://crates.io/crates/multiconst)
[![api-docs](https://docs.rs/multiconst/badge.svg)](https://docs.rs/multiconst/*)

For destructuring an expression into multiple constants.

The primary feature of this crate is the [`multiconst`] macro, 
which destructuring an expression into multiple constants.

# Example

For more examples you can look [in the docs for `multiconst`][multiconst-examples]

# Basic

This example demonstrates destructuring an array (whose length is inferred) 
into multiple constants.

```rust
use multiconst::multiconst;

assert_eq!(A, 0b11);
assert_eq!(B, 0b111);
assert_eq!(C, 0b1111);
assert_eq!(D, 0b11111);

multiconst!{
    pub const [A, B, C, D]: [u64; _] = mersennes_from(2);
}

/// Generates all mersenne numbers (binary numbers that are all `1` bits)
/// from `start` amount of 1s up to `start + N - 1`.
const fn mersennes_from<const N: usize>(start: u32) -> [u64; N] {
    let mut out = [0; N];
    multiconst::for_range!{i in 0..N =>
        out[i] = (1 << (i as u32 + start)) - 1;
    }
    out
}


```

# No-std support

`multiconst` is `#![no_std]`, it can be used anywhere Rust can be used.

# Minimum Supported Rust Version

`multiconst` requires Rust 1.51.0, requiring crate features to use newer language features.


[`multiconst`]: https://docs.rs/multiconst/latest/multiconst/macro.multiconst.html
[multiconst-examples]: https://docs.rs/multiconst/latest/multiconst/macro.multiconst.html#examples