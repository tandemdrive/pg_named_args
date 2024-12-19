extern crate self as pg_named_args;

pub use pg_named_args_macros::{fragment, query_args};

#[derive(Clone, Copy, Default)]
pub struct Fragment(&'static str);

impl Fragment {
    pub fn get(self) -> &'static str {
        self.0
    }

    #[doc(hidden)]
    /// This is the constructor used by the [fragment!] macro.
    /// It is not intended to be used manually.
    pub const fn new_unchecked(sql: &'static str) -> Self {
        Self(sql)
    }
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
