/// type-level representation of an identifier,
///
/// `C` is expected to be a tuple of [`TChars`].
///
/// This is the type that [`field_name`] expands to when it's passed an identifier.
///
/// # Representation
///
/// Identifiers are represented as `TIdent` parameterized with a
/// tuple of `TChars`. `TChars` represents  8-`char`-long chunks in the identifier.
///
/// # Example
///
/// ### Representation
///
/// ```rust
/// use multiconst::{TIdent, TChars, field_name};
///
/// let _: field_name!(foo) =
///     TIdent::<(
///         TChars<'f', 'o', 'o', ' ', ' ', ' ', ' ', ' '>,
///     )>::NEW;
///
/// let _: field_name!(outright) =
///     TIdent::<(
///         TChars<'o', 'u', 't', 'r', 'i', 'g', 'h', 't'>,
///     )>::NEW;
///
///
/// let _: field_name!(adventure) =
///     TIdent::<(
///         TChars<'a', 'd', 'v', 'e', 'n', 't', 'u', 'r'>,
///         TChars<'e', ' ', ' ', ' ', ' ', ' ', ' ', ' '>,
///     )>::NEW;
///
///
/// ```
///
/// [`field_name`]: crate::field_name
pub struct TIdent<C>(core::marker::PhantomData<C>);

impl<C> TIdent<C> {
    /// Constructs a `TIdent`
    pub const NEW: Self = Self(core::marker::PhantomData);
}

/// Type-level representation of up to 8 characters,
/// with spaces padding the const arguments after the last character.
///
/// [`TIdent`] describes how this is used.
pub struct TChars<
    const C0: char,
    const C1: char,
    const C2: char,
    const C3: char,
    const C4: char,
    const C5: char,
    const C6: char,
    const C7: char,
>;

/// A type-level usize, used to query the type of positional fields
/// (tuple field s).
///
/// # Examples
///
/// ### `FieldType` implementation
///
/// This example demonstrates how to make tuple structs easy to destructure in the
/// [`multiconst`](crate::multiconst) macro without derives.
///
/// ```rust
/// use multiconst::{multiconst, FieldType, Usize};
///
/// multiconst!{
///     const Foo(DIR, LENGTH): Foo = Foo(Direction::Left, 123);
/// }
///
/// assert_eq!(DIR, Direction::Left);
/// assert_eq!(LENGTH, 123);
///
///
/// struct Foo(Direction, u8);
///
/// impl FieldType<Usize<0>> for Foo {
///     type Type = Direction;
/// }
///
/// impl FieldType<Usize<1>> for Foo {
///     type Type = u8;
/// }
///
/// #[derive(Debug, PartialEq)]
/// enum Direction {
///     Left,
///     Right,
///     Up,
///     Down,
/// }
///
/// ```
///
///
pub struct Usize<const N: usize>;
