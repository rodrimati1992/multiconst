use multiconst::{FieldType, Usize};


multiconst::multiconst!{
    const Range{start}: Range<u8> = 3..5;
}

multiconst::multiconst!{
    const Range{start:}: Range<u8> = 3..5;
}

multiconst::multiconst!{
    const Range{start: B:}: Range<u8> = 3..5;
}

multiconst::multiconst!{
    const Range{start: BB:,}: Range<u8> = 3..5;
}

multiconst::multiconst!{
    const Range{start: C, .., end: D}: Range<u8> = 3..5;
}



struct TupledPart(u8, bool);

impl FieldType<Usize<0>> for TupledPart {
    type Type = u8;
}
impl FieldType<Usize<1>> for TupledPart {
    type Type = bool;
}

multiconst::multiconst!{
    const TupledPart(A:): TupledPart = TupledPart(10, false);
}

multiconst::multiconst!{
    const TupledPart(A:,): TupledPart = TupledPart(10, false);
}

multiconst::multiconst!{
    const TupledPart(A, .., B): TupledPart = TupledPart(10, false);
}




fn main(){}

