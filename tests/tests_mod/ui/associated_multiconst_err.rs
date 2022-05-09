
// ensuring that the macro doesn't work outside of inherent impls.
multiconst::associated_multiconst!{
    const [A, B]: [u32; 2] = [3, 5];
}

fn main(){}