#[cfg(feature = "__ui_tests")]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/tests_mod/ui/*err.rs");
    t.pass("tests/tests_mod/ui/*fine.rs");
}
