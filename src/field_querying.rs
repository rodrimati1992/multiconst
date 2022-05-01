mod field_type;

pub use self::field_type::{FieldType, GetFieldType};

/*
For future use when structs are supported:

/// type-level representation of an identifier,
///
/// `C` is expected to be a tuple of [`Chars`].
pub struct TIdent<C>(core::marker::PhantomData<C>);


/// Represents up to 8 characters,
/// with spaces padding the const arguments after the last character.
pub struct Chars<
    const C0: char,
    const C1: char,
    const C2: char,
    const C3: char,
    const C4: char,
    const C5: char,
    const C6: char,
    const C7: char,
>;
*/

// type-level usize
pub struct Usize<const N: usize>;

///////////////////////////////////////////////////////////////////////////
