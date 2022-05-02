/// Macro that expands to a type-level representation of a field name.
///
/// This expands to one of these:
/// - [`TIdent`]
/// - [`Usize`]
///
///
#[macro_export]
macro_rules! field {
    ($($args:tt)*) => {
        $crate::__::__priv_field_proc_macro!{
            $crate

            $($args)*
        }
    };
}
