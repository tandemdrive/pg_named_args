//! This library allows one to use named arguments in PostgreSQL queries. This
//! library is especially aimed at supporting
//! [rust-postgres](https://github.com/sfackler/rust-postgres). A macro is provided
//! to rewrite queries with named arguments, into a query and its positional
//! arguments.
//!
//! # Dependencies
//! The macro expands to usage of `postgres-types`, so make sure to have it in your dependencies:
//! ```toml
//! [dependencies]
//! postgres-types = ...
//! pg_named_args = ...
//! ```
//!
//! # Query Argument Syntax
//! The macro uses struct syntax for the named arguments.
//! The struct name `Args` is required to support rustfmt and rust-analyzer.
//! As can be seen from the example below, shorthand field initialization is also allowed for named arguments.
//!
//! ```
//! # use pg_named_args::query_args;
//! # struct Period {
//! #     start: u32,
//! #     end: u32,
//! # }
//! #
//! let location = "netherlands";
//! let period = Period {
//!     start: 2020,
//!     end: 2030,
//! };
//!
//! let (query, args) = query_args!(
//!     r"
//!     SELECT location, time, report
//!     FROM weather_reports
//!     WHERE location = $location
//!         AND time BETWEEN $start AND $end
//!     ORDER BY location, time DESC
//!     ",
//!     Args {
//!         location,
//!         start: period.start,
//!         end: period.end,
//!     }
//! );
//! ```
//! ```ignore
//! let rows = client.query(query, args).await?;
//! ```
//!
//! # Insert Syntax
//! For `INSERT`'s a special syntax is supported, which helps to avoid mismatches
//! between the list of column names and the values:
//!
//! ```
//! # use pg_named_args::query_args;
//! #
//! let location = "sweden";
//! let time = "monday";
//! let report = "sunny";
//!
//! let (query, args) = query_args!(
//!     r"
//!     INSERT INTO weather_reports
//!         ( $[location, time, report] )
//!     VALUES
//!         ( $[..] )
//!     ",
//!     Args {
//!         location,
//!         time,
//!         report
//!     }
//! );
//! ```
//! ```ignore
//! client.execute(query, args).await?;
//! ```
//!
//! # IDE Support
//!
//! First, the syntax used by this macro is compatible with rustfmt.
//! Run rustfmt as you would normally and it will format the macro.
//!
//! Second, the macro is implemented in a way that is rust-analyzer "friendly".
//! This means that rust-analyzer knows which arguments are required and can complete them.
//! Use the code action "Fill struct fields" or ask rust-analyzer to complete a field name.

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
