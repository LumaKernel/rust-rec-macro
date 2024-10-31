use rec_macro::rec;

fn main() {
    rec! {
        // #[memo_vec]
        // #[memo_map]
        #[memo_set]
        async fn collatz(x: u64) {
            println!("enter: {}", x);
            if x <= 1 {
                return;
            }
            if x % 2 == 0 {
                collatz(x / 2).await;
            } else {
                collatz(x * 3 + 1).await;
            }
        }
    }

    println!("start 200");
    collatz(200);
    println!("start 30");
    collatz(30);
    println!("start 90");
    collatz(90);
}
