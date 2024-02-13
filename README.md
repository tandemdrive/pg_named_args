# PostgreSQL named arguments

<!-- cargo-rdme start -->

This library allows one to use named arguments in PostgreSQL queries. This
library is especially aimed at supporting
[rust-postgres](https://github.com/sfackler/rust-postgres). A macro is provided
to rewrite queries with named arguments, into a query and its positional
arguments.

## Dependencies
The macro expands to usage of `postgres-types`, so make sure to have it in your dependencies:
```toml
[dependencies]
postgres-types = ...
pg_named_args = ...
```

## Query Argument Syntax
The macro uses struct syntax for the named arguments.
The struct name `Args` is required to support rustfmt and rust-analyzer.
As can be seen from the example below, shorthand field initialization is also allowed for named arguments.

```rust
let location = "netherlands";
let period = Period {
    start: 2020,
    end: 2030,
};

let (query, args) = query_args!(
    r"
    SELECT location, time, report
    FROM weather_reports
    WHERE location = $location
        AND time BETWEEN $start AND $end
    ORDER BY location, time DESC
    ",
    Args {
        location,
        start: period.start,
        end: period.end,
    }
);
```
```rust
let rows = client.query(query, args).await?;
```

## Insert Syntax
For `INSERT`'s a special syntax is supported, which helps to avoid mismatches
between the list of column names and the values:

```rust
let location = "sweden";
let time = "monday";
let report = "sunny";

let (query, args) = query_args!(
    r"
    INSERT INTO weather_reports
        ( $[location, time, report] )
    VALUES
        ( $[..] )
    ",
    Args {
        location,
        time,
        report
    }
);
```
```rust
client.execute(query, args).await?;
```

## IDE Support

First, the syntax used by this macro is compatible with rustfmt.
Run rustfmt as you would normally and it will format the macro.

Second, the macro is implemented in a way that is rust-analyzer "friendly".
This means that rust-analyzer knows which arguments are required and can complete them.
Use the code action "Fill struct fields" or ask rust-analyzer to complete a field name.

<!-- cargo-rdme end -->

## Goals

- Increase usability of executing PostgreSQL queries from Rust.

- Reduce the risk of mismatching query arguments.

- Support for rustfmt to help with formatting.

- Support for rust-analyzer completion and some code actions.

## Contributing

We welcome community contributions to this project.

Please read our [Contributor Terms](CONTRIBUTING.md#contributor-terms) before
you make any contributions.

Any contribution intentionally submitted for inclusion, shall comply with the
Rust standard licensing model (MIT OR Apache 2.0) and therefore be dual licensed
as described below, without any additional terms or conditions:

### License

This contribution is dual licensed under EITHER OF

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
