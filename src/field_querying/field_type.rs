use super::Usize;

use crate::utils_for_macros::SeqLength;

/// For querying the type of a field in `Self`.
///
/// The name of the field is represented with the `Name` type parameter
///
/// The type of nested fields can be queried by passing a tuple of field names.
///
/// You can use the [`GetFieldType`] type alias as a more convenient way to
/// get the `Type` associated type.
///
pub trait FieldType<Name> {
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

macro_rules! impl_nested_field_type {
    (
        $((
            $(($bounded_ty:ident $name:ident $field_ty:ident))*,
            $last_field_ty:ident
        ))*
    ) => {
        $(
            impl<$($bounded_ty, $name,)* $last_field_ty>
                FieldType<($($name,)*)>
            for T0
            where
                $($bounded_ty: FieldType<$name, Type = $field_ty>,)*
            {
                type Type = $last_field_ty;
            }
        )*
    };
}

impl<T> FieldType<()> for T {
    type Type = T;
}

/*
fn main(){
    let per_line = 6;
    for len in 1..=8 {
        print!("(");
        for i in 0..len {
            if i % per_line == 0 && len >= per_line {
                print!("\n    ");
            }
            print!("(T{0} N{0} T{1}) ", i, i + 1);
        }
        print!(", ");
        if len >= per_line { print!("\n    ") }
        print!("T{}", len);
        if len >= per_line { print!("\n") }
        println!(")");
    }
}

*/

impl_nested_field_type! {
    ((T0 N0 T1) , T1)
    ((T0 N0 T1) (T1 N1 T2) , T2)
    ((T0 N0 T1) (T1 N1 T2) (T2 N2 T3) , T3)
    ((T0 N0 T1) (T1 N1 T2) (T2 N2 T3) (T3 N3 T4) , T4)
    ((T0 N0 T1) (T1 N1 T2) (T2 N2 T3) (T3 N3 T4) (T4 N4 T5) , T5)
    (
        (T0 N0 T1) (T1 N1 T2) (T2 N2 T3) (T3 N3 T4) (T4 N4 T5) (T5 N5 T6) ,
        T6
    )
    (
        (T0 N0 T1) (T1 N1 T2) (T2 N2 T3) (T3 N3 T4) (T4 N4 T5) (T5 N5 T6)
        (T6 N6 T7) ,
        T7
    )
    (
        (T0 N0 T1) (T1 N1 T2) (T2 N2 T3) (T3 N3 T4) (T4 N4 T5) (T5 N5 T6)
        (T6 N6 T7) (T7 N7 T8) ,
        T8
    )
}

/////////////////////////////////////////////////////////////////////////////
//                     FieldType impls

/// Does not check that `I` is inside the array.
impl<T, const I: usize, const N: usize> FieldType<Usize<I>> for [T; N] {
    type Type = T;
}

/*

fn main(){
    let per_line = 8;
    for len in 1..=8 {
        print!("(");
        for i in 0..len {
            if i % per_line == 0 && len >= per_line {
                print!("\n    ");
            }
            print!("(T{0} {0}) ", i);
        }
        print!(", ");
        if len >= per_line { print!("\n    ") }
        print!("(");
        for i in 0..len {
            print!("T{0},", i);
        }
        print!("), ");
        print!("{}", len);
        if len >= per_line { print!("\n") }
        println!(")");
    }
}
*/

macro_rules! tuple_impls {
    (
        $((
            $(($out:ident $index:literal))*,
            $tuple_ty:tt,
            $len:tt
        ))*
    ) => {
        $(
            $(
                tuple_impls__field_type!{
                    ($out $index)
                    $tuple_ty
                }
            )*

            tuple_impls__seq_length!{ $tuple_ty $tuple_ty $len }
        )*
    };
}

macro_rules! tuple_impls__field_type {
    (
        ($out:ident $index:literal)
        ($($tparam:ident,)*)
    ) => {
        impl<$($tparam,)*> FieldType<Usize<$index>> for ($($tparam,)*) {
            type Type = $out;
        }
    }
}

macro_rules! tuple_impls__seq_length {
    ( ($($tparam:ident,)*) $tuple_ty:tt $len:tt ) => {
        impl<$($tparam,)*> SeqLength for $tuple_ty {
            const LENGTH: usize = $len;
        }
    }
}

tuple_impls! {
    ((T0 0) , (T0,), 1)
    ((T0 0) (T1 1) , (T0,T1,), 2)
    ((T0 0) (T1 1) (T2 2) , (T0,T1,T2,), 3)
    ((T0 0) (T1 1) (T2 2) (T3 3) , (T0,T1,T2,T3,), 4)
    ((T0 0) (T1 1) (T2 2) (T3 3) (T4 4) , (T0,T1,T2,T3,T4,), 5)
    ((T0 0) (T1 1) (T2 2) (T3 3) (T4 4) (T5 5) , (T0,T1,T2,T3,T4,T5,), 6)
    ((T0 0) (T1 1) (T2 2) (T3 3) (T4 4) (T5 5) (T6 6) , (T0,T1,T2,T3,T4,T5,T6,), 7)
    (
        (T0 0) (T1 1) (T2 2) (T3 3) (T4 4) (T5 5) (T6 6) (T7 7) ,
        (T0,T1,T2,T3,T4,T5,T6,T7,), 8
    )
}
