use rec_macro::rec;
use rec_macro::*;
use std::cell::RefCell;
use std::rc::Rc;

struct BiasedMemo<T> {
    base: T,
    bias: u64,
}
impl<Output, T: Memo<u64, Output>> Memo<u64, Output> for BiasedMemo<T> {
    fn get_memo(&self, args: &u64) -> Option<Output> {
        self.base.get_memo(&(args + self.bias))
    }
    fn insert_memo(&mut self, args: u64, output: &Output) {
        self.base.insert_memo(args + self.bias, output)
    }
}

fn main() {
    let memo = Rc::new(RefCell::new(Vec::new()));
    rec! {
        #[memo(Rc::clone(&memo))]
        async fn is_odd(x: u64) -> bool {
            println!("enter: is_odd({})", x);
            if x == 0 {
                false
            } else {
                !is_odd(x - 1).await
            }
        }
    }

    let memo1 = BiasedMemo {
        base: Rc::clone(&memo),
        bias: 1,
    };
    rec! {
        #[memo(memo1)]
        async fn is_even(x: u64) -> bool {
            println!("enter: is_even({})", x);
            if x == 0 {
                true
            } else {
                !is_even(x - 1).await
            }
        }
    }

    dbg!(is_odd(4));
    dbg!(is_even(4));
}
