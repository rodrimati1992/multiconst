use core::cmp::Ordering;

use multiconst::{multiconst, FieldType, Usize};

mod fp {
    multiconst::field_name_aliases! {
        pub a,
        pub b,
        pub c,
        pub d,
    }
}

#[test]
fn destructure_range_types() {
    use core::ops::{Range, RangeFrom, RangeTo, RangeToInclusive};

    {
        multiconst! {
            const ::core::ops::Range {start: S, end: E}: Range<u8> = 3..5;
        }

        assert_eq!(S, 3u8);
        assert_eq!(E, 5u8);
    }
    {
        multiconst! {
            const core::ops::RangeFrom {start: S}: RangeFrom<u8> = 3..;
        }

        assert_eq!(S, 3u8);
    }
    {
        multiconst! {
            const core::ops::RangeTo<u8> {end: E}: RangeTo<u8> = ..5;
        }

        assert_eq!(E, 5u8);
    }
    {
        multiconst! {
            const RangeToInclusive{end: E}: RangeToInclusive<u8> = ..=5;
        }

        assert_eq!(E, 5u8);
    }
}

#[test]
fn destructure_other_std_types() {
    use core::{cmp::Reverse, num::Wrapping};

    {
        multiconst! {
            const Reverse([A, B, C]): Reverse<[u8; 3]> = Reverse([3, 5, 8]);
        }
        assert_eq!(A, 3u8);
        assert_eq!(B, 5u8);
        assert_eq!(C, 8u8);
    }
    {
        multiconst! {
            const Wrapping((A, B, C)): Wrapping<(u8, u16, u32)> = Wrapping((3u8, 5u16, 8u32));
        }
        assert_eq!(A, 3u8);
        assert_eq!(B, 5u16);
        assert_eq!(C, 8u32);
    }
}

#[test]
fn destructure_braced() {
    struct Braced {
        a: u8,
        b: Ordering,
        c: &'static str,
        d: bool,
    }

    impl FieldType<fp::a> for Braced {
        type Type = u8;
    }
    impl FieldType<fp::b> for Braced {
        type Type = Ordering;
    }
    impl FieldType<fp::c> for Braced {
        type Type = &'static str;
    }
    impl FieldType<fp::d> for Braced {
        type Type = bool;
    }

    const DEF_BR: Braced = Braced {
        a: 3,
        b: Ordering::Equal,
        c: "foo",
        d: true,
    };

    {
        multiconst! {
            const Braced{a: A, b: B, c: C, d: D}: Braced = DEF_BR;
        }

        assert_eq!(A, DEF_BR.a);
        assert_eq!(B, DEF_BR.b);
        assert_eq!(C, DEF_BR.c);
        assert_eq!(D, DEF_BR.d);
    }

    // ignore some fields
    {
        multiconst! {
            const Braced{a: A, b: B, ..}: Braced = DEF_BR;
        }

        assert_eq!(A, DEF_BR.a);
        assert_eq!(B, DEF_BR.b);
    }

    // ignore some fields, with a trailing comma
    {
        multiconst! {
            const Braced{a: A, b: B, ..,}: Braced = DEF_BR;
        }

        assert_eq!(A, DEF_BR.a);
        assert_eq!(B, DEF_BR.b);
    }

    // ignore some fields, with `_`
    {
        multiconst! {
            const Braced{a: A, b: _, c: _, d: D}: Braced = DEF_BR;
        }

        assert_eq!(A, DEF_BR.a);
        assert_eq!(D, DEF_BR.d);
    }

    // reorder the fields
    {
        multiconst! {
            const Braced{
                d: D,
                c: C,
                a: A,
                b: B,
            }: Braced = DEF_BR;
        }

        assert_eq!(A, DEF_BR.a);
        assert_eq!(B, DEF_BR.b);
        assert_eq!(C, DEF_BR.c);
        assert_eq!(D, DEF_BR.d);
    }

    // annotate the field types
    {
        multiconst! {
            const Braced {
                d: D,
                c: C: &'static str,
                a: A,
                b: B: Ordering,
            }: Braced = DEF_BR;
        }

        assert_eq!(A, DEF_BR.a);
        assert_eq!(B, DEF_BR.b);
        assert_eq!(C, DEF_BR.c);
        assert_eq!(D, DEF_BR.d);
    }
}

#[test]
fn destructure_braced_partial() {
    struct BracedPart {
        a: u8,
        b: Ordering,
        c: &'static str,
        d: bool,
    }

    impl FieldType<fp::a> for BracedPart {
        type Type = u8;
    }
    impl FieldType<fp::b> for BracedPart {
        type Type = Ordering;
    }

    const DEF_BRP: BracedPart = BracedPart {
        a: 3,
        b: Ordering::Equal,
        c: "foo",
        d: true,
    };

    {
        multiconst! {
            const BracedPart{a: A, b: B, ..}: BracedPart = DEF_BRP;
        }

        assert_eq!(A, DEF_BRP.a);
        assert_eq!(B, DEF_BRP.b);
    }

    // ensure that ignored fields don't need a type annotation
    {
        multiconst! {
            const BracedPart{a: A, b: B, c: _, d: _}: BracedPart = DEF_BRP;
        }

        assert_eq!(A, DEF_BRP.a);
        assert_eq!(B, DEF_BRP.b);
    }

    {
        multiconst! {
            const BracedPart{
                a: A,
                b: B,
                c: C: &'static str,
                d: D: bool,
            }: BracedPart = DEF_BRP;
        }

        assert_eq!(A, DEF_BRP.a);
        assert_eq!(B, DEF_BRP.b);
        assert_eq!(C, DEF_BRP.c);
        assert_eq!(D, DEF_BRP.d);
    }
}

#[test]
fn tuple_destructuring() {
    struct Tupled(u8, Ordering, &'static str, bool);

    impl FieldType<Usize<0>> for Tupled {
        type Type = u8;
    }
    impl FieldType<Usize<1>> for Tupled {
        type Type = Ordering;
    }
    impl FieldType<Usize<2>> for Tupled {
        type Type = &'static str;
    }
    impl FieldType<Usize<3>> for Tupled {
        type Type = bool;
    }

    const DEF_TUP: Tupled = Tupled(3, Ordering::Equal, "foo", true);

    {
        multiconst! {
            const Tupled(A, B, C, D): Tupled = DEF_TUP;
        }

        assert_eq!(A, DEF_TUP.0);
        assert_eq!(B, DEF_TUP.1);
        assert_eq!(C, DEF_TUP.2);
        assert_eq!(D, DEF_TUP.3);
    }
    {
        multiconst! {
            const Tupled(A, _, _, D): Tupled = DEF_TUP;
        }

        assert_eq!(A, DEF_TUP.0);
        assert_eq!(D, DEF_TUP.3);
    }
    {
        multiconst! {
            const Tupled(A, B, C: &'static str, D: bool): Tupled = DEF_TUP;
        }

        assert_eq!(A, DEF_TUP.0);
        assert_eq!(B, DEF_TUP.1);
        assert_eq!(C, DEF_TUP.2);
        assert_eq!(D, DEF_TUP.3);
    }
    {
        multiconst! {
            const Tupled(A, B, ..): Tupled = DEF_TUP;
        }

        assert_eq!(A, DEF_TUP.0);
        assert_eq!(B, DEF_TUP.1);
    }
}

#[test]
fn tuple_part_destructuring() {
    struct TupledPart(u8, Ordering, &'static str, bool);

    impl FieldType<Usize<0>> for TupledPart {
        type Type = u8;
    }
    impl FieldType<Usize<1>> for TupledPart {
        type Type = Ordering;
    }

    const DEF_PTUP: TupledPart = TupledPart(3, Ordering::Equal, "foo", true);

    {
        multiconst! {
            const TupledPart(A, B, C: &'static str, D: bool): TupledPart = DEF_PTUP;
        }

        assert_eq!(A, DEF_PTUP.0);
        assert_eq!(B, DEF_PTUP.1);
        assert_eq!(C, DEF_PTUP.2);
        assert_eq!(D, DEF_PTUP.3);
    }

    // ensure that ignored fields don't need a type annotation
    {
        multiconst! {
            const TupledPart(A, B, _, _): TupledPart = DEF_PTUP;
        }

        assert_eq!(A, DEF_PTUP.0);
        assert_eq!(B, DEF_PTUP.1);
    }

    // ensure that ignored fields don't need a type annotation
    {
        multiconst! {
            const TupledPart(A, B, ..): TupledPart = DEF_PTUP;
        }

        assert_eq!(A, DEF_PTUP.0);
        assert_eq!(B, DEF_PTUP.1);
    }

    // trailing comma after ..
    {
        multiconst! {
            const TupledPart(A, B, ..,): TupledPart = DEF_PTUP;
        }

        assert_eq!(A, DEF_PTUP.0);
        assert_eq!(B, DEF_PTUP.1);
    }
}
