#[cfg(not(feature = "__no_ui_tests"))]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/tests_mod/ui/*err.rs");

    #[cfg(feature = "derive")]
    t.compile_fail("tests/tests_mod/ui_derive/*err.rs");
}
