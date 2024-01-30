# PostgreSQL named arguments

This library allows one to use named arguments in PostgreSQL queries. This
library is especially aimed at supporting
[rust-postgres](https://github.com/sfackler/rust-postgres). A macro is provided
to rewrite queries with named arguments, into a query and its positional
arguments.

Example:

```rust
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
let rows = client.query(query, args).await?;
```

The macro uses struct syntax for the named arguments. The struct name `Args` is required to support rustfmt and rust-analyzer.
As can be seen from the example above, shorthand field initialization is also allowed for named arguments.

For `INSERT`'s a special syntax is supported, which helps to avoid mismatches
between the list of column names and the values:

```rust
let (query, args) = query_args!(
    r"
    INSERT INTO weather_reports
        ( $[location, time, report] ) 
    VALUES 
        ( $[..] )
    ",
    Args { location, time, report }
);
client.execute(query, args).await?;
```

The macro is written in a way that is rust-analyzer friendly.
This means that rust analyzer knows which parameters are required and can complete them.
Use the code action "Fill struct fields" or ask rust analyzer to complete a field name.

## Goals

- Increase usability of executing PostgreSQL queries from Rust.

- Reduce the risk of mismatching query arguments.

- Support for rust-analyzer completion and some code actions.

- Support for rustfmt to help with formatting.

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
