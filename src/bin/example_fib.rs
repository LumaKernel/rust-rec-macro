use rec_macro::rec;

fn main() {
    rec! {
        async fn fib(n: u64) -> u64 {
            match n {
                0 => 0,
                1 => 1,
                n => (fib(n - 2).await + fib(n - 1).await) % 107,
            }
        }
    }

    dbg!(fib(20));
    assert_eq!(fib(20), 6765 % 107);
    // assert_eq!(fib(40), 102334155 % 107); // Slow!
}
