// ensure that .. patterns can't be used with inferred array lengths
multiconst::multiconst!{
    const [A100, ..]: [u32; _] = [3, 5, 8];
}


// ensure that .. patterns can't be used multiple times in arrays
multiconst::multiconst!{
    const [.., A100, ..]: [u32; _] = [3, 5, 8];
}


// ensure that F @ .. patterns can't be used in tuples
multiconst::multiconst!{
    const (A100, A @ ..): (u32, u32, u32, u32) = (3, 5, 8, 13);
}

// ensure that .. patterns can't be used multiple times in tuples
multiconst::multiconst!{
    const (.., A100, ..): (u32, u32, u32, u32) = (3, 5, 8, 13);
}


fn main(){}