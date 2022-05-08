#![no_std]
#![deny(unused_must_use)]
#![forbid(unsafe_code)]

extern crate alloc;

#[cfg(not(any(test, feature = "derive")))]
extern crate proc_macro as used_proc_macro;

#[cfg(any(test, feature = "derive"))]
extern crate proc_macro2 as used_proc_macro;

#[cfg(any(feature = "__dbg", test))]
extern crate std;

#[cfg(feature = "derive")]
mod derive_macro;

mod error;
mod for_field_macros;
mod for_multiconst_macro;
mod parsing;
mod pattern;
mod pattern_processing;
mod syntax;
mod type_;
mod utils;

#[cfg(test)]
mod test_utils;

use crate::error::Error;

#[proc_macro]
pub fn __priv_multiconst_proc_macro(args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    crate::for_multiconst_macro::macro_impl(args.into())
        .unwrap_or_else(|e| e)
        .into()
}

#[proc_macro]
pub fn __priv_field_proc_macro(args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    crate::for_field_macros::field_macro_impl(args.into())
        .unwrap_or_else(|e| e)
        .into()
}

#[proc_macro]
pub fn __priv_field_name_aliases_proc_macro(
    args: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    crate::for_field_macros::field_name_aliases_macro_impl(args.into())
        .unwrap_or_else(|e| e)
        .into()
}

#[cfg(feature = "derive")]
#[proc_macro_derive(FieldType, attributes(field_type))]
pub fn field_type_derive(args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ret = crate::derive_macro::derive_macro_impl(args.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into();

    ret
}
