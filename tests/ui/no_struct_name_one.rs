use pg_named_args::query_args;

fn main() {
    let test = 4;
    query_args!("$a, $b", { test });
}
