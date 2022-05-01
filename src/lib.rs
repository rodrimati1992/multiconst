//! Reserving for use *soon* (at most finished during may 2022)
//!
//!
//!
//!

///
pub mod field_querying;

mod macros;
mod utils_for_macros;

#[doc(hidden)]
pub mod __ {
    pub use multiconst_proc_macros::__priv_multiconst_proc_macro;

    pub use crate::{
        field_querying::{GetFieldType, Usize},
        utils_for_macros::{AssertSameTypes, SeqLength, Type},
    };

    pub use core::{compile_error, primitive::usize};
}
