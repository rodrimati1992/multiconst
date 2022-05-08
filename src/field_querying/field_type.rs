/// For querying the type of a field in `Self`.
///
/// The name of the field is represented with the `Name` type parameter
///
/// The type of nested fields can be queried by passing a tuple of field names.
///
/// You can derive this tarit with the [`FieldType`](derive@crate::FieldType)
/// derive (requires the "derive" feature).
///
/// You can use the [`GetFieldType`] type alias as a more convenient way to
/// get the `Type` associated type.
///
/// # Examples
///
/// <span id = "manual-impl-example"></span>
/// ### Manual implementation
///
/// This example demonstrates how to make structs easy to destructure in the
/// [`multiconst`](crate::multiconst) macro without derives.
///
/// ```rust
/// use multiconst::{field_name, multiconst, FieldType};
///
/// multiconst!{
///     const Foo{name: NAME, length: LENGTH}: Foo = Foo {
///         name: "hello",
///         length: 123,
///     };
/// }
///
/// assert_eq!(NAME, "hello");
/// assert_eq!(LENGTH, 123);
///
///
/// struct Foo {
///     name: &'static str,
///     length: usize,
/// }
///
/// impl FieldType<field_name!(name)> for Foo {
///     type Type = &'static str;
/// }
///
/// impl FieldType<field_name!(length)> for Foo {
///     type Type = usize;
/// }
///
/// ```
///
/// ### Derived
///
/// Using the [`FieldType`](derive@crate::FieldType) derive to impl this trait.
///
/// TODO
///
pub trait FieldType<Names> {
    /// The type of the field.
    type Type;
}

/// Gets the type of a (potentially nested) field.
///
/// The type of nested fields can be queried by passing a tuple of field names.
///
/// # Examples
///
/// ### Type alias
///
/// Gets the type of a field in a type alias
///
/// ```rust
/// use multiconst::{GetFieldType, Usize};
///
/// type Foo = (u8, u16, u32, u64, u128);
///
/// let _elem0: GetFieldType<Foo, Usize<0>> = 3u8;
/// let _elem1: GetFieldType<Foo, Usize<1>> = 5u16;
/// let _elem2: GetFieldType<Foo, Usize<2>> = 8u32;
///
/// ```
///
/// ### Nested field type
///
/// This demonstrates how the type of a nested field is queried.
///
/// ```rust
/// use multiconst::{GetFieldType, Usize};
///
/// type Foo = ([u32; 2], (u64, &'static str));
///
/// let _elem_0_0: GetFieldType<Foo, (Usize<0>, Usize<0>)> = 3u32;
/// let _elem_1_0: GetFieldType<Foo, (Usize<1>, Usize<0>)> = 5u64;
/// let _elem_1_1: GetFieldType<Foo, (Usize<1>, Usize<1>)> = "hello";
///
/// ```
pub type GetFieldType<This, Names> = <This as FieldType<Names>>::Type;
