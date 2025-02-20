#![allow(unknown_lints)] // TODO: remove this when 1.85 is stable
#![deny(
    clippy::literal_string_with_formatting_args,
    unknown_lints, // enable the lint again
    clippy::inconsistent_struct_constructor,
    clippy::empty_structs_with_brackets,
    unreachable_code
)]
use pg_named_args::{fragment, query_args};

#[test]
fn query_args_should_support_identifiers_as_values() {
    let b = "Fred";
    let c = "Flintstone";
    let (query, params) = query_args!(
        r"
INSERT INTO fred_flintstone(a, b, c)
VALUES(true, $b, $c);
            ",
        Args { b, c }
    );

    let expected_query = r"
INSERT INTO fred_flintstone(a, b, c)
VALUES(true, $1, $2);
            ";
    assert_eq!(query.trim(), expected_query.trim());
    assert_eq!(params.len(), 2);
}

#[test]
fn query_args_should_support_key_value_pairs_as_values() {
    let b = 37_i64;
    let c = 42_i64;
    let (query, params) = query_args!(
        r"
INSERT INTO fred_flintstone(a, b, c)
VALUES(true, $b, $c);
            ",
        Args { c, b }
    );
    let expected_query = r"
INSERT INTO fred_flintstone(a, b, c)
VALUES(true, $1, $2);
            ";
    assert_eq!(query.trim(), expected_query.trim());
    assert_eq!(params.len(), 2);
}

#[test]
fn query_args_should_support_list_syntax() {
    let b = 37_i64;
    let c = 42_i64;
    let (query, params) = query_args!(
        r"
INSERT INTO fred_flintstone(a, $[b, c])
VALUES(true, $[..]);
            ",
        Args { b, c }
    );
    let expected_query = r"
INSERT INTO fred_flintstone(a, b, c)
VALUES(true, $1, $2);
            ";
    assert_eq!(query.trim(), expected_query.trim());
    assert_eq!(params.len(), 2);
}

#[test]
fn query_args_should_support_multiple_substitutions() {
    let b = 37_i64;
    let c = 42_i64;
    let (query, params) = query_args!(
        r"
INSERT INTO fred_flintstone(a, b, c)
VALUES(true, $b, $c)
ON CONFLICT DO UPDATE SET b = $b WHERE c = $c;
            ",
        Args { b, c }
    );
    let expected_query = r"
INSERT INTO fred_flintstone(a, b, c)
VALUES(true, $1, $2)
ON CONFLICT DO UPDATE SET b = $1 WHERE c = $2;
            ";
    assert_eq!(query.trim(), expected_query.trim());
    assert_eq!(params.len(), 2);
}

#[test]
fn query_args_should_accept_fragment() {
    let a = fragment!("test_fragment");
    let (query, args) = query_args!(
        "$xx, ${a}", // this looks like a format string to trigger clippy::literal_string_with_formatting_args
        Sql { a },
        Args { xx: 1 }
    );
    let expected_query = r"$1, test_fragment";
    assert_eq!(query.trim(), expected_query);
    assert_eq!(args.len(), 1);
}
