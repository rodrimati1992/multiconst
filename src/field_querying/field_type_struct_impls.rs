use crate::FieldType;

mod name {
    use crate::Usize;

    crate::field_name_aliases! {
        pub start,
        pub end,
    }

    pub type N0 = Usize<0>;
}

macro_rules! impl_for_struct {
    (
        impl $impl_args:tt $type:ty {
            $( $field_name:ident: $field_ty:ty),*
            $(,)?
        }
    ) => {
        $(
            impl_for_field!{
                impl $impl_args $type;
                $field_name $field_name $field_ty
            }
        )*
    };
}

macro_rules! impl_for_field{
    (
        impl[$($impl_args:tt)*] $type:ty;
        $field_name:tt $path:tt $field_ty:ty
    ) => {
        impl<$($impl_args)*> FieldType<name::$path> for $type {
            type Type = $field_ty;
        }
    }
}

impl_for_struct! {
    impl[T] core::ops::Range<T> {
        start: T,
        end: T,
    }
}

impl_for_struct! {
    impl[T] core::ops::RangeFrom<T> {
        start: T,
    }
}

impl_for_struct! {
    impl[T] core::ops::RangeTo<T> {
        end: T,
    }
}

impl_for_struct! {
    impl[T] core::ops::RangeToInclusive<T> {
        end: T,
    }
}

impl_for_struct! {
    impl[T] core::cmp::Reverse<T> { N0: T }
}

impl_for_struct! {
    impl[T] core::num::Wrapping<T> { N0: T }
}
