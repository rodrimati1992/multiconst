type Unit = ();

// ensure that mismatched tuple pattern and tuple type alias type error at macro expansion time
multiconst::multiconst!{
    const (A300, A301): Unit = ();
}

fn main(){}