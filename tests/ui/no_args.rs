use pg_named_args::query_args;

fn main() {
    query_args!("$a, $b");
    query_args!("${a}");
}
