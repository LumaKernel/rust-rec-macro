use rec_macro::rec;

fn main() {
    rec! {
        async fn gcd(a: u64, b: u64) -> u64 {
            if b == 0 {
                a
            } else {
                gcd(b, a % b).await
            }
        }
    }

    dbg!(gcd(1893, 1742));
}
