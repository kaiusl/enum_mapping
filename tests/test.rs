
#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/errors.rs");
    t.pass("tests/basic.rs");
}