use core::marker::PhantomData;

pub trait SeqLength {
    const LENGTH: usize;
}

impl<T, const N: usize> SeqLength for [T; N] {
    const LENGTH: usize = N;
}

// remaining impls for other tuples in field_type.rs
impl SeqLength for () {
    const LENGTH: usize = 0;
}

/// For asserting that Self and Self::Type are the same type.
pub trait TypeIdentity {
    type Type: ?Sized;
}

impl<T: ?Sized> TypeIdentity for T {
    type Type = T;
}

pub struct AssertSameTypes<A: ?Sized, B: ?Sized>(
    PhantomData<(fn() -> PhantomData<A>, fn() -> PhantomData<B>)>,
)
where
    A: TypeIdentity<Type = B>;

/// usable for:
/// - assigning multiple spans to any type
/// - transforming a type into a path
pub type Type<T> = T;
