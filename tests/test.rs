
#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/no_to.rs");
    t.pass("tests/basic.rs");
}