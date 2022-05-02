//! For destructuring an expression into multiple constants.
//!
//! The primary feature of this crate is the [`multiconst`] macro,
//! which destructuring an expression into multiple constants.
//!
//! # Example
//!
//! For more examples you can look [in the docs for `multiconst`][multiconst-examples]
//!
//! # Basic
//!
//! This example demonstrates destructuring an array (whose length is inferred)
//! into multiple constants.
//!
//! ```rust
//! use multiconst::multiconst;
//!
//! assert_eq!(A, 0b11);
//! assert_eq!(B, 0b111);
//! assert_eq!(C, 0b1111);
//! assert_eq!(D, 0b11111);
//!
//! multiconst!{
//!     pub const [A, B, C, D]: [u64; _] = mersennes_from(2);
//! }
//!
//! /// Generates all mersenne numbers (binary numbers that are all `1` bits)
//! /// from `start` amount of 1s up to `start + N - 1`.
//! const fn mersennes_from<const N: usize>(start: u32) -> [u64; N] {
//!     let mut out = [0; N];
//!     multiconst::for_range!{i in 0..N =>
//!         out[i] = (1 << (i as u32 + start)) - 1;
//!     }
//!     out
//! }
//!
//!
//! ```
//!
//!
//! # No-std support
//!
//! `multiconst` is `#![no_std]`, it can be used anywhere Rust can be used.
//!
//! # Minimum Supported Rust Version
//!
//! `multiconst` requires Rust 1.51.0, requiring crate features to use newer language features.
//!
//!
//! [`multiconst`]: crate::multiconst
//! [multiconst-examples]: crate::multiconst#examples

#![no_std]
#![forbid(unsafe_code)]

mod field_querying;

pub use crate::field_querying::*;

mod macros;
mod utils_for_macros;

#[doc(hidden)]
pub mod __ {
    pub use multiconst_proc_macros::{
        __priv_field_name_aliases_proc_macro, __priv_field_proc_macro, __priv_multiconst_proc_macro,
    };

    pub use crate::{
        field_querying::{GetFieldType, TChars, TIdent, Usize},
        utils_for_macros::{AssertSameTypes, SeqLength, Type},
    };

    pub use core::{compile_error, ops::Range, primitive::usize};
}
