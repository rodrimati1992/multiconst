/**
Derives the [`FieldType`] trait for a struct.

# Generated code

By default this derive generates impls of [`FieldType`] for all fields, with a condition.

The condition is that a field is one of:
- A non-`pub` field in a non-`pub` struct
- A `pub` field in a `pub` struct

"non-`pub`" includes `pub(crate)` and smaller visibilities.

Whether [`FieldType`] is implemented for a field can be overridden with
the [`#[field_type(pub)]`](#pub-attr) and
[`#[field_type(priv)]`](#priv-attr) attributes.

# Container Attributes

Attributes that go above the struct.

### `#[field_type(crate = foo::bar::baz)]`

This attribute overrides the path to `multiconst`,
for when the `multiconst` crate is reexported at some path.

# Container or field Attributes

Attributes that can go above the struct and fields.

<span id = "pub-attr"></span>
### `#[field_type(pub)]`

Tells the macro to generate a [`FieldType`] impl for the field.

When this is used above the struct,
it tells the macro to generate impls of [`FieldType`] for all fields.

When this is used on a field,
it tells the macro to generate an impl of [`FieldType`] for that field,
overriding [`#[field_type(priv)]`](#priv-attr) attributes on the struct.

<span id = "priv-attr"></span>
### `#[field_type(priv)]`

Tells the macro to **not** generate a [`FieldType`] impl for the field.

When this is used above the struct,
it tells the macro to not generate impls of [`FieldType`] for any fields.

When this is used on a field,
it tells the macro to **not** generate an impl of [`FieldType`] for that field,
overriding [`#[field_type(pub)]`](#pub-attr) attributes on the struct.


# Examples

TODO




[`FieldType`]: trait@crate::FieldType
*/
#[cfg(feature = "derive")]
pub use multiconst_proc_macros::FieldType;
