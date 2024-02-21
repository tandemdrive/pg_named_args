use pg_named_args::query_args;

fn main() {
    let _: (_, &[u32]) = query_args!(
        "$a, $b",
        Args {
            ..Args {
                a: "test",
                b: "test"
            }
        }
    );
}
