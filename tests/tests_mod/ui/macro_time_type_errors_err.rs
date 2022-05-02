// ensure that mismatched syntactic tuple pattern are type error at macro expansion time
multiconst::multiconst!{
    const (A200, A201): () = ();
}

// ensure that mismatched syntactic tuple pattern are type errors
multiconst::multiconst!{
    const (): (u32, u32) = ();
}

// ensure that array patterns don't work with tuple types
multiconst::multiconst!{
    const [A]: (u32,) = [10];
}

// ensure that tuple patterns don't work with array types
multiconst::multiconst!{
    const (A,): [u32; 1] = [10];
}







fn main(){}