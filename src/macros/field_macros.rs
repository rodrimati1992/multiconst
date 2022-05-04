/// Macro that expands to a type-level representation of a field name.
///
/// This macro can be passed as the `Names` argument of
/// [`FieldType`] and [`GetFieldType`], as the name of the field.
///
/// This expands to one of these:
/// - [`TIdent`](crate::TIdent)
/// - [`Usize`](crate::Usize)
///
/// # Examples
///
/// For an example of manually implementing [`FieldType`]
/// you can [look here](crate::FieldType#manual-impl-example).
///
/// ### Querying a field's type
///
/// ```rust
/// use multiconst::{GetFieldType, field_name};
///
/// {
///     type RU = std::ops::Range<usize>;
///
///     let _: GetFieldType<RU, field_name!(start)> = 100usize;
///     let _: GetFieldType<RU, field_name!(end)> = 100usize;
/// }
///
/// {
///     type TUP = (&'static str, Option<u8>);
///
///     let _: GetFieldType<TUP, field_name!(0)> = "hello";
///     let _: GetFieldType<TUP, field_name!(1)> = Some(10u8);
/// }
///
///
/// ```
///
/// [`FieldType`]: crate::FieldType
/// [`GetFieldType`]: crate::GetFieldType
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
///
/// The aliases can be passed as the `Names` argument of
/// [`FieldType`] and [`GetFieldType`], as the name of the field.
///
/// # Examples
///
/// ### Querying a field's type
///
/// ```rust
/// use multiconst::GetFieldType;
///
/// mod names {
///     multiconst::field_name_aliases!{
///         // equivalent to `pub(super) type F0 = field_name!(0);`
///         pub(super) F0 = 0,
///
///         // equivalent to `pub(crate) type F1 = field_name!(1);`
///         pub(crate) F1 = 1,
///
///         // equivalent to `pub type start = field_name!(start);`
///         pub start,
///
///         // equivalent to `pub type End = field_name!(end);`
///         pub End = end,
///     }
/// }
///
/// {
///     type RU = std::ops::Range<char>;
///
///     let _: GetFieldType<RU, names::start> = 'a';
///     let _: GetFieldType<RU, names::End> = 'b';
/// }
///
/// {
///     type TUP = (&'static str, Option<u8>);
///
///     let _: GetFieldType<TUP, names::F0> = "hello";
///     let _: GetFieldType<TUP, names::F1> = Some(10u8);
/// }
///
/// ```
///
///
/// [`FieldType`]: crate::FieldType
/// [`GetFieldType`]: crate::GetFieldType
///
#[macro_export]
macro_rules! field_name_aliases {
    ($($args:tt)*) => {
        $crate::__::__priv_field_name_aliases_proc_macro!{
            $crate

            $($args)*
        }
    };
}
