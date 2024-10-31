use rec_macro::rec;

fn main() {
    rec! {
        #[memo_map]
        async fn fact(x: usize) -> u64 {
            if x == 0 {
                1
            } else {
                x as u64 * fact(x - 1).await % 1000000007
            }
        }
    }

    // 457992974
    dbg!(fact(100000));
    for _ in 0..100000 {
        assert_eq!(fact(100000), 457992974);
    }
}
