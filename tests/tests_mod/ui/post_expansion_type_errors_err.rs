type Pair = (u32, u32);


// ensure that the type mismatch errors point to the expression.
multiconst::multiconst!{
    const A000: u32 = ();
}


// ensure that mismatched tuple pattern and tuple type alias type error at macro expansion time
multiconst::multiconst!{
    const (): Pair = (3, 3);
}


// ensure that too long array patterns error 
multiconst::multiconst!{
    const [A1, B1, C1]: [(); 2] = [(),()];
}

// ensure that too long array patterns error 
multiconst::multiconst!{
    const [A11, B11, C11, ..]: [(); 2] = [(),()];
}

// ensure that too short array patterns error 
multiconst::multiconst!{
    const [A2, B2, C2]: [(); 4] = [(),(),(),()];
}



multiconst::multiconst!{
    const [X, Z, Z]: [(); 3] = [(), (), ()];
}

multiconst::multiconst!{
    const Z: () = ();
}



// ensure that type annotations cause a type mismatch when they should
multiconst::multiconst!{
    const _: () = 100;
}

// ensure that type annotations cause a type mismatch when they should
multiconst::multiconst!{
    const std::ops::Range{start: _: u32, ..}: std::ops::Range<u8> = 3u8..5;
}

// ensure that type annotations cause a type mismatch when they should
multiconst::multiconst!{
    const std::ops::Range{start: [_, _]: [u32; 2], ..}: std::ops::Range<[u8; 2]> = 
        [0u8; 2]..[0u8; 2];
}


fn main(){}