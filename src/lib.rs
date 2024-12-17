extern crate self as pg_named_args;

pub use pg_named_args_macros::query_args;

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
