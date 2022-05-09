This is the changelog, summarising changes in each version(some minor changes may be ommited).

# 0.2

### 0.2.2

Added struct pattern support in `multiconst`.

Added `FieldType` derive macro for structs, enabled with the `"derive"` crate feature.

Added `associated_multiconst` to destructure an expression into multiple associated constants.

Added `TIdent` and `TChars` marker types to represent identifiers on the type level.

Added `field_name` and `field_name_aliases` macros to get a type-level representation of a field name.

Added `"derive"` crate feature.

Added `syn`, `proc-macro2`, and `quote` dependencies when the `"derive"` feature is enabled.



### 0.2.0

Declared the `multiconst` macro, for destructuring an expression into multiple constants. With support for array, tuple, identifier, underscore, and remainder patterns.

Declared the `for_range` macro.

Declared the `Usize` struct, to represent a positional field name.

Declared the `FieldType` trait, for querying the type of a field.

Declared the `GetFieldType` type alias, for querying the type of a field.

Implemented `FieldType` for arrays of all lengths, and tuples up to 8 elements long.

Implemented `FieldType` for accessing fields up to 8 levels deep.

Added the `multiconst_proc_macros` 0.2.0 proc macro dependency (this is only relevant on platforms where proc macros can't be used)