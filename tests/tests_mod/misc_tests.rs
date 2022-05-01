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
