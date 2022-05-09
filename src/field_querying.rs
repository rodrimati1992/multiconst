//! Types and traits for querying the types of fields/elements.

mod field_name;
mod field_type;
mod field_type_prim_impls;
mod field_type_struct_impls;

pub use self::{
    field_name::{TChars, TIdent, Usize},
    field_type::{FieldType, GetFieldType},
};

///////////////////////////////////////////////////////////////////////////
