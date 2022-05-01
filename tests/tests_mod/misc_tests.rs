use multiconst::multiconst;

multiconst! {
    const _: _ = 100;

    const _: u32 = 100;
}

multiconst! {
    const FOO: &'static str = "hello";
}

#[test]
fn single_ident() {
    assert_eq!(FOO, "hello");
}

#[test]
fn attribute_application_test() {
    {
        multiconst! {
            #[cfg(any())] const [#[cfg(all())] A, B]: [u32; _] = [3, 5];
        }
        const A: () = ();
        const B: &str = "nope";

        assert_eq!(A, ());
        assert_eq!(B, "nope");
    }
    {
        multiconst! {
            #[cfg(all())] const [#[cfg(any())] A, B]: [u32; _] = [3, 5];
        }
        const A: () = ();

        assert_eq!(A, ());
        assert_eq!(B, 5);
    }
}
