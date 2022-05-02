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


fn main(){}