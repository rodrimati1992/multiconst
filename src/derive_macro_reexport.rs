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

[example that uses this attribute](#crate-attr-example)

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

[example that uses this attribute](#vis-example)

<span id = "priv-attr"></span>
### `#[field_type(priv)]`

Tells the macro to **not** generate a [`FieldType`] impl for the field.

When this is used above the struct,
it tells the macro to not generate impls of [`FieldType`] for any fields.

When this is used on a field,
it tells the macro to **not** generate an impl of [`FieldType`] for that field,
overriding [`#[field_type(pub)]`](#pub-attr) attributes on the struct.

[example that uses this attribute](#vis-example)

# Examples

### Basic

```rust
use multiconst::{FieldType, multiconst};

multiconst!{
    // structs inhibit length inference for array fields,
    // which means that `Pairs<_>` doesn't work.
    pub const Pairs([(A, B), (C, D)]): Pairs<2> = Pairs([(3, 33), (5, 55)]);
}

assert_eq!(A, 3);
assert_eq!(B, 33);
assert_eq!(C, 5);
assert_eq!(D, 55);


#[derive(FieldType)]
struct Pairs<const L: usize>([(u32, u32); L]);

```

### Bitflags usecase

```rust
use multiconst::{FieldType, multiconst};

multiconst!{
    pub const WithFlagNames{
        /// The destructured constants can be documented like this
        flag_names: FLAG_NAMES,
        bitflags: BITS,
    }: WithFlagNames<2> = WithFlagNames::from_indices([1, 4]);
}

assert_eq!(FLAG_NAMES, ["B", "E"]);
assert_eq!(BITS, 0b0110_0011);



#[derive(FieldType)]
pub struct WithFlagNames<const L: usize> {
    pub flag_names: [&'static str; L],
    pub bitflags: u32,
}

impl<const L: usize> WithFlagNames<L> {
    pub const fn from_indices(indices: [usize; L]) -> Self {
        let mut flag_names = [""; L];
        let mut bitflags = 0u32;

        multiconst::for_range!{i in 0..L =>
            let (flag_name, bits) = FLAGS[indices[i]];
            flag_names[i] = flag_name;
            bitflags |= bits;
        }

        Self{flag_names, bitflags}
    }
}


const FLAGS: [(&'static str, u32); 5] = [
    ("A", 0b0000_0001),
    ("B", 0b0000_0011),
    ("C", 0b0000_0100),
    ("D", 0b0001_0000),
    ("E", 0b0110_0000),
];

```

<span id = "vis-example"></span>
### Override visibility

This example demonstrates the `#[field_type(pub)]` and `#[field_type(priv)]` attributes.

```rust
use multiconst::{FieldType, multiconst};

multiconst!{
    const Flags{
        is_large: LARGE,
        
        // since there's no FieldType impl for the field, it requires a type annotation.
        is_long: LONG: bool,

        is_heavy: HEAVY,
    }: Flags = {
        Flags {
            is_large: true,
            is_long: false,
            is_heavy: true,
        }
    };
}

assert_eq!(LARGE, true);
assert_eq!(LONG, false);
assert_eq!(HEAVY, true);


#[derive(FieldType)]
pub struct Flags {
    pub is_large: bool,
    
    // this attribute is required to prevent implementing `FieldType` for this field.
    #[field_type(priv)]
    pub is_long: bool,
    
    // this attribute is required to destructure non-pub fields in pub structs.
    #[field_type(pub)]
    pub(crate) is_heavy: bool,
}

```

<span id = "crate-attr-example"></span>
### Reexport example

This example demonstrates how this derive can be used when reexported.

```rust
# pub extern crate multiconst as mc;
# mod reexport { pub use ::mc as multiconst; }
# fn main(){
use reexport::multiconst::{FieldType, multiconst};

multiconst! {
    const Foo(A, B): Foo = Foo(3, 5);
}

assert_eq!(A, 3);
assert_eq!(B, 5);


#[derive(FieldType)]
#[field_type(crate = reexport::multiconst)]
struct Foo(u32, u32);

# } 
```




[`FieldType`]: trait@crate::FieldType
*/
#[cfg(feature = "derive")]
pub use multiconst_proc_macros::FieldType;
