use rec_macro::rec;
use std::collections::HashMap;

fn main() {
    rec! {
        #[memo(HashMap::<_, _>::new())]
        async fn fib(n: u64) -> u64 {
            match n {
                0 => 0,
                1 => 1,
                n => (fib(n - 2).await + fib(n - 1).await) % 107,
            }
        }
    }

    assert_eq!(fib(20), 6765 % 107);
    assert_eq!(fib(40), 102334155 % 107);
    assert_eq!(fib(1000000), 86);
    dbg!(fib(1000000));
}
