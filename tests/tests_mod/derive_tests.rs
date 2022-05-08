use core::cmp::Ordering;

use static_assertions::assert_not_impl_all;

use multiconst::{multiconst, FieldType, Usize};

mod fp {
    multiconst::field_name_aliases! {
        pub c,
        pub d,
    }
}

#[test]
fn derive_destructure_braced() {
    #[derive(FieldType)]
    struct Braced {
        a: u8,
        b: Ordering,
        c: &'static str,
        d: bool,
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
fn derive_destructure_braced_partial() {
    #[derive(FieldType)]
    pub struct BracedPart {
        pub a: u8,
        #[field_type(pub)]
        b: Ordering,
        #[field_type(priv)]
        pub c: &'static str,
        d: bool,
    }

    assert_not_impl_all! {BracedPart: FieldType<fp::c>, FieldType<fp::d>}

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
fn derive_tuple_destructuring() {
    #[derive(FieldType)]
    struct Tupled(u8, Ordering, &'static str, bool);

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
fn derive_tuple_part_destructuring() {
    #[derive(FieldType)]
    pub struct TupledPart(
        pub u8,
        #[field_type(pub)] Ordering,
        #[field_type(priv)] pub &'static str,
        bool,
    );

    assert_not_impl_all! {TupledPart: FieldType<Usize<2>>, FieldType<Usize<3>>}

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
