/// Destructures a constant expression into multiple *associated* constants.
///
/// This macro is identical to [`multiconst`],
/// except that it's for declaring inherent associated constants.
///
/// # Examples
///
/// All the [`multiconst` examples](crate::multiconst#examples) work with this macro,
/// below are additional examples to show this macro specifically.
///
/// ### Usage
///
/// ```rust
/// # use multiconst::field_name;
/// use multiconst::{FieldType, associated_multiconst};
///
///
/// struct GetProperty<T>(T);
///
/// impl<T> GetProperty<T> {
///     associated_multiconst!{
///         pub const TypeProperties{
///             /// The size of `T`.
///             size: SIZE,
///             /// The alignment of `T`.
///             align: ALIGN,
///             /// Whether `T` has any dropping code.
///             needs_drop: NEEDS_DROP,
///         }: TypeProperties = TypeProperties::new::<T>();
///     }
/// }
///
/// assert_eq!(GetProperty::<[u64; 2]>::SIZE, 16);
/// assert_eq!(GetProperty::<[u64; 2]>::ALIGN, std::mem::align_of::<u64>());
/// assert_eq!(GetProperty::<[u64; 2]>::NEEDS_DROP, false);
///
///
///
/// // this derive requires the "derive" feature
/// # /*
/// #[derive(FieldType)]
/// # */
/// struct TypeProperties {
///     size: usize,
///     align: usize,
///     needs_drop: bool,
/// }
/// #
/// # impl FieldType<field_name!(size)> for TypeProperties { type Type = usize; }
/// # impl FieldType<field_name!(align)> for TypeProperties { type Type = usize; }
/// # impl FieldType<field_name!(needs_drop)> for TypeProperties { type Type = bool; }
/// #
///
/// impl TypeProperties {
///     const fn new<T>() -> Self {
///         Self {
///             size: std::mem::size_of::<T>(),
///             align: std::mem::align_of::<T>(),
///             needs_drop: std::mem::needs_drop::<T>(),
///         }
///     }
/// }
///
/// ```
///
///
///
///
#[macro_export]
macro_rules! associated_multiconst {
    ($($args:tt)*) => {
        $crate::__::__priv_associated_multiconst_proc_macro!{
            $crate

            $($args)*
        }
    };
}
