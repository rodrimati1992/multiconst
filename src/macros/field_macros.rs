/// Macro that expands to a type-level representation of a field name.
///
/// This expands to one of these:
/// - [`TIdent`]
/// - [`Usize`]
///
///
#[macro_export]
macro_rules! field_name {
    ($($args:tt)*) => {
        $crate::__::__priv_field_proc_macro!{
            $crate

            $($args)*
        }
    };
}

/// Declares type aliases for type-level representations of field names.
#[macro_export]
macro_rules! field_name_aliases {
    ($($args:tt)*) => {
        $crate::__::__priv_field_name_aliases_proc_macro!{
            $crate

            $($args)*
        }
    };
}
