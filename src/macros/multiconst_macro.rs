/**
Destructures a constant expression into multiple constants

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
- `$(#[$battr:meta])* $binding:ident`: a binding pattern,
which destructures that part of the pattern into a `$binding` constant.
- `_`: an ignore pattern, most useful inside other patterns
- `$(#[$battr:meta])* $binding:ident @ ..`(only usable in arrays):
destructures the rest of the matched array into a `$binding` constant.
- `..`(only usable in arrays or tuples): ignores the rest of the elements in the matched collection.
- `[ $($array_elem:`[`pattern`](#pattern)`),* $(,)? ]`: an array pattern
- `( $($tuple_elem:`[`pattern`](#pattern)`),* )`: a tuple pattern
- `( $pattern:`[`pattern`](#pattern)` )`: a parenthesized pattern

`$vis:vis` can be any visibility modifier,
it is then used as the visibility of every generated constant.

`$type:ty` can be any type (so long as it's compatible with the pattern).

`$value:expr` can be any const expression (so long as its type is `$type`).

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

This macro only supports destructuring tuples and arrays,
it will support destructuring structs in a future release.

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
