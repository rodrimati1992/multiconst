use core::ops::Range;

use multiconst::{FieldType, Usize};

multiconst::multiconst!{
    const Range{start: A:u16, ..}: Range<u8> = 3..5;
}

struct Tupled(u8);

impl FieldType<Usize<0>> for Tupled {
    type Type = u8;
}

multiconst::multiconst!{
    const Tupled(B: u32): Tupled = Tupled(10);
}


fn main(){}