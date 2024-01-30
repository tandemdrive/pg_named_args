use pg_named_args::query_args;

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
