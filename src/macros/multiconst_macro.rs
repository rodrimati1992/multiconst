/**
Destructures a constant expression into multiple constants

For destructuring into *associated* constants you can use [`associated_multiconst`]

For examples [look here](#examples)

# Syntax

This uses a macro_rules-like syntax to describe the parameters.

The input syntax for `multiconst` is
```text
$(
    $(#[$attr:meta])*
    $vis:vis const $pattern:pattern : $type:ty = $value:expr;
)*
```

<span id = "pattern"></span>
Where `:pattern` arguments can be any of:

- binding pattern: `$(#[$battr:meta])* $binding:ident`:
which destructures that part of the pattern into a `$binding` constant.

- ignore pattern: `_`: most useful inside other patterns

- remainder pattern: `$(#[$battr:meta])* $binding:ident @ ..` (only usable in arrays):
destructures the rest of the matched array into a `$binding` constant.

- ignore remainder pattern: `..` (usable in arrays, structs, or tuples):
ignores the rest of the elements in the matched collection.

- array pattern: `[ $($array_elem:`[`pattern`](#pattern)`),* $(,)? ]`

- tuple pattern: `( $($tuple_elem:`[`pattern`](#pattern)`),* )`:

- struct pattern:
```text
$struct_name:path {
    $(
        $(#[$fattr:meta])*
        $field_name:field_name: $struct_elem:pattern $(: $type_annotation:ty)?
    ),*
    $(, ..)?
    $(,)?
}
```

- tuple struct pattern:
```text
$struct_name:path (
    $(
        $struct_elem:pattern $(: $type_annotation:ty)?
    ),*
    $(, ..)?
    $(,)?
)
```

- `( $pattern:`[`pattern`](#pattern)` )`: a parenthesized pattern

`$vis:vis` can be any visibility modifier,
it is then used as the visibility of every generated constant.

`$type:ty` can be any type (so long as it's compatible with the pattern).

`$value:expr` can be any const expression (so long as its type is `$type`).

`$field_name:field_name` can be either an untyped integer literal or an identifier.

### Attributes

Attributes used on patterns are copied to the generated constant,
which allows documenting `pub` constants.

[example of how to do that here](#attrs-example)

### Struct patterns

Structs patterns (by default) require the struct to implement
the [`FieldType`][trait@crate::FieldType] trait to query the types of destructured fields.
You can annotate the types of fields to avoid the
[`FieldType`][trait@crate::FieldType] requirement.

Struct patterns inhibit length inference of array type arguments,
so you must annotate the array's length.

[example of struct patterns](#example-struct)

# Type Inference

This macro has a limited form of type inference,
it can infer the length of array types written syntactically as an array type
(the length of array type aliases can't be inferred).

Note that `..` patterns in arrays are incompatible with inferring the length of
that array's type.

You don't need to explicitly declare the type of ignored patterns
in syntactic tuple and array types
(type aliases do require explicit types though),
which means that this is allowed:
```rust
# multiconst::multiconst!{
    const (A, .., B): (u32, _, _, _, u64) = foo();
# }
#
# const fn foo() -> (u32, (), (), (), u64) {
#       (0, (), (), (), 1)
# }
```

# Limitations

This macro only supports destructuring tuples, structs, and arrays.

There are no plans to support destructuring slices or enums.

# Examples

### Array destructuring

This example demonstrates array length inference
(this macro uses the amount of elements in the pattern as the length)

```rust
use multiconst::{for_range, multiconst};

multiconst! {
    const [P0, P1, _, P2, P3, _, P4]: [u128; _] = powers_of_two();
}

assert_eq!(P0, 1);
assert_eq!(P1, 2);
assert_eq!(P2, 8);
assert_eq!(P3, 16);
assert_eq!(P4, 64);


const fn powers_of_two<const LEN: usize>() -> [u128; LEN] {
    let mut arr = [0; LEN];

    for_range!{ i in 0usize..LEN =>
        arr[i] = 1 << i;
    }

    arr
}
```

### Remainder pattern

This example demonstrates the `FOO @ ..` pattern,
to get the rest of the elements of an array.


```
use multiconst::{for_range, multiconst};

type Arr = [u8; 8];

multiconst! {
    const [ELEM0, ELEM1, REM @ .., END]: Arr = [3, 5, 7, 11, 13, 17, 19, 23];
}

assert_eq!(ELEM0, 3);
assert_eq!(ELEM1, 5);
assert_eq!(REM, [7, 11, 13, 17, 19]);
assert_eq!(END, 23);

```

### Pseudo-Random number generation

This example demonstrates tuple destructuring

```rust
use multiconst::multiconst;

multiconst! {
    const ([A, B, C, D], NEXT_SEED): ([u32; _], u32) = rng(100);
}

const fn rng<const N: usize>(seed: u32) -> ([u32; N], u32) {
    // implementation hidden
    # let mut arr = [0u32; N];
    # let mut i = 0usize;
    #
    # while i != N {
    #   arr[i] = seed.wrapping_add(i as u32);
    #
    #   i += 1;
    # }
    #
    # (arr, seed.wrapping_add(N as u32))
}

```

<span id = "example-struct"></span>
### Struct example

This example demonstrates struct destructuring.

*/
#[cfg_attr(feature = "derive", doc = "```rust")]
#[cfg_attr(not(feature = "derive"), doc = "```ignore")]
/**
use multiconst::{FieldType, multiconst};

use std::ops::Range;

{
    multiconst!{
        // Structs that impl/derive `FieldType` can be destructured like this
        const Deriving{
            foo: F,
            bar: B,
        }: Deriving = Deriving{foo: 10, bar: 20};
    }

    assert_eq!(F, 10);
    assert_eq!(B, 20);

    // The `FieldType` derive requires the `"derive"` feature.
    #[derive(FieldType)]
    struct Deriving {
        foo: u32,
        bar: u64,
    }
}

{
    multiconst!{
        // Structs that don't impl `FieldType` can be destructured by annotating field types
        const NonDeriving{
            baz: B: u32,
            qux: Q: u64,
        }: NonDeriving = NonDeriving{baz: 10, qux: 20};
    }

    assert_eq!(B, 10);
    assert_eq!(Q, 20);

    struct NonDeriving {
        baz: u32,
        qux: u64,
    }
}


```



<span id = "attrs-example"></span>
### Attributes example

This example demonstrates how attributes (mostly documentation comments)
can be used in multiconst.

```rust
use multiconst::multiconst;

use std::ops::Range;


multiconst! {
    /// Attributes (like documentation attributes) can go here,
    /// they are copied below the attributes used on inner patterns.
    pub const (
        /// documentation on the pattern are copied to the generated constants.
        MAJOR,
        MINOR,
        PATCH,
    ): (u32, u32, u32) = VERSIONS;
}

assert_eq!(MAJOR, 1);
assert_eq!(MINOR, 2);
assert_eq!(PATCH, 3);

# const VERSIONS: (u32, u32, u32) = (1, 2, 3);


multiconst! {
    pub const Range {
        /// Documentation on struct field patterns looks like this,
        /// these doc attributes are for the `S` constant.
        start: S,
        ..
    }: Range<u32> = 3..5;
}

assert_eq!(S, 3);
```


*/
#[macro_export]
macro_rules! multiconst {
    ($($args:tt)*) => {
        $crate::__::__priv_multiconst_proc_macro!{
            $crate

            $($args)*
        }
    };
}
