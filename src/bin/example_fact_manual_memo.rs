use core::cell::RefCell;
use rec_macro::rec;
use std::collections::HashMap;

fn main() {
    let memo = RefCell::new(HashMap::<u64, u64>::new());
    rec! {
        async fn fact_manual_memo(x: u64) -> u64 {
            if let Option::Some(e) = memo.borrow_mut().get(&x) {
                return *e;
            }
            let r = {
                if x == 0 {
                    1
                } else {
                    x * fact_manual_memo(x - 1).await % 1000000007
                }
            };
            memo.borrow_mut().insert(x, r);
            r
        }
    }

    // 457992974
    dbg!(fact_manual_memo(100000));
}
