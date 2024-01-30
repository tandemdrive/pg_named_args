# PostgreSQL named arguments

This library allows one to use named arguments in PostgreSQL queries. This
library is especially aimed at supporting
[rust-postgres](https://github.com/sfackler/rust-postgres). A macro is provided
to rewrite queries with named arguments, into a query and its positional
arguments.

Example:

```rust
let (query, args) = pg_named_args!(
    r"
    SELECT location, time, report
    FROM weather_reports
    WHERE location = $location
        AND time BETWEEN $start AND $end
    ORDER BY location, time DESC
    ",
    {
        location,
        "start": &period.start,
        "end": &period.end,
    }
);
let rows = client.query($query, $args).await?;
```

As can be seen from the example above a shortcut is allowed when the name
of the argument is identical to the variable name, similar to what's allowd in
Rust `struc`'s.

For `INSERT`'s a special syntax is supported, which helps to avoid mismatches
between the list of column names and the values:

```rust
let (query, args) = pg_named_args!(
    r"
    INSERT INTO weather_reports
    ($[
        location,
        time,
        report
    ]) VALUES ($[..])
    ",
    { location, time, report }
);
client.execute($query, $args).await?;
```

## Goals

- Increase usability of executing PostgreSQL queries from Rust.

- Reduce the risk of mismatching query arguments.

- Basic support for using rust-analyzer to help one with the syntax.

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
