//! Types and traits for querying the types of fields/elements.

mod field_type;
mod field_type_struct_impls;

pub use self::field_type::{FieldType, GetFieldType};

/// type-level representation of an identifier,
///
/// `C` is expected to be a tuple of [`TChars`].
pub struct TIdent<C>(core::marker::PhantomData<C>);

/// Represents up to 8 characters,
/// with spaces padding the const arguments after the last character.
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

/// Represents a type-level usize, used to query the type of positional fields
/// (tuple fields).
pub struct Usize<const N: usize>;

///////////////////////////////////////////////////////////////////////////
