#[macro_export]
macro_rules! rec {
    (
        #[memo_vec]
        $($tt:tt)*
    ) => {
        rec! {
            #[memo(::std::vec::Vec::<_>::new())]
            $($tt)*
        }
    };
    (
        #[memo_map]
        $($tt:tt)*
    ) => {
        rec! {
            #[memo(::std::collections::HashMap::<_, _>::new())]
            $($tt)*
        }
    };
    (
        #[memo_hashmap]
        $($tt:tt)*
    ) => {
        rec! {
            #[memo(::std::collections::HashMap::<_, _>::new())]
            $($tt)*
        }
    };
    (
        #[memo_btreemap]
        $($tt:tt)*
    ) => {
        rec! {
            #[memo(::std::collections::BTreeMap::<_, _>::new())]
            $($tt)*
        }
    };
    (
        #[memo_set]
        $($tt:tt)*
    ) => {
        rec! {
            #[memo(::std::collections::HashSet::<_, _>::new())]
            $($tt)*
        }
    };
    (
        #[memo_hashset]
        $($tt:tt)*
    ) => {
        rec! {
            #[memo(::std::collections::HashSet::<_, _>::new())]
            $($tt)*
        }
    };
    (
        #[memo_btreeset]
        $($tt:tt)*
    ) => {
        rec! {
            #[memo(::std::collections::BTreeSet::<_, _>::new())]
            $($tt)*
        }
    };
    (
        #[memo($memo:expr)]
        async fn $name:ident($($arg:ident : $arg_type:ty),*) -> $ret_type:ty {
            $($body:tt)*
        }
    ) => {
        let $name = {
            #[allow(unused_parens)]
            $crate::Rec::<_, ($($arg_type),*), $ret_type, _, _>::new(|$name, ($($arg),*): ($($arg_type),*)| {
                let $name = $crate::ForceMover(($name, ($($arg),*)));
                #[warn(unused_parens)]
                async {
                    let $name = $name;
                    let $name = $name.0;
                    #[allow(unused_parens)]
                    let ($name, ($($arg),*)) = $name;
                    #[allow(unused_variables)]
                    let $name = |$($arg:$arg_type),*| unsafe { (*$name)(($($arg),*)) };
                    $($body)*
                }
            }, $memo)
        };
        let $name = unsafe { ::core::pin::Pin::new_unchecked(&$name) };
        let $name = ($name,);
        let $name = ($name.0, |$($arg : $arg_type),*| unsafe { $name.0.call($name.0.me(), ($($arg),*)) });
        let $name = $name.1;
    };

    (
        #[memo($($memo:tt)*)]
        async fn $name:ident($($arg:ident : $arg_type:ty),*)  {
            $($body:tt)*
        }
    ) => {
        rec! {
            #[memo($($memo)*)]
            async fn $name($($arg : $arg_type),*) -> () {
                $($body)*
            }
        }
    };

    (
        async fn $name:ident($($arg:ident : $arg_type:ty),*) $(-> $ret_type:ty)? {
            $($body:tt)*
        }
    ) => {
        rec! {
            #[memo($crate::NoMemo {})]
            async fn $name($($arg : $arg_type),*) $(-> $ret_type)? {
                $($body)*
            }
        }
    };
}

fn waker_do_nothing_vtable() -> &'static ::core::task::RawWakerVTable {
    unsafe fn clone_raw(data: *const ()) -> ::core::task::RawWaker {
        ::core::task::RawWaker::new(data, waker_do_nothing_vtable())
    }
    unsafe fn wake_raw(_data: *const ()) {}
    unsafe fn wake_by_ref_raw(_data: *const ()) {}
    unsafe fn drop_raw(_data: *const ()) {}
    &::core::task::RawWakerVTable::new(clone_raw, wake_raw, wake_by_ref_raw, drop_raw)
}
fn new_waker_do_nothing() -> ::core::task::Waker {
    let raw_waker = ::core::task::RawWaker::new(::core::ptr::null(), waker_do_nothing_vtable());
    unsafe { ::core::task::Waker::from_raw(raw_waker) }
}

struct PopperInner<Args, T> {
    // TODO: Use bare type.
    waker: ::core::option::Option<::core::task::Waker>,
    res: ::core::option::Option<T>,
    args: ::core::option::Option<Args>,
}
// Generally unsafe. Only usage from Rec is safe.
pub struct Popper<Args, T> {
    // This realizes virtual pinning.
    // Internal status is only below, no need for real pinning.
    inner: *const ::std::vec::Vec<PopperInner<Args, T>>,
}
impl<Args, T> ::core::future::Future for Popper<Args, T> {
    type Output = T;
    fn poll(
        self: ::core::pin::Pin<&mut Self>,
        cx: &mut ::core::task::Context<'_>,
    ) -> ::core::task::Poll<T> {
        let inner_vec = unsafe { &mut *(self.inner as *mut ::std::vec::Vec<PopperInner<Args, T>>) };

        let inner = unsafe {
            #[allow(invalid_reference_casting)]
            &mut *(inner_vec.last().unwrap_unchecked() as *const _ as *mut PopperInner<Args, T>)
        };
        if let ::core::option::Option::Some(res) = inner.res.take() {
            // unchecked pop
            unsafe {
                debug_assert!(!inner_vec.is_empty());
                inner_vec.set_len(inner_vec.len() - 1);
                drop(::core::ptr::read(inner_vec.as_ptr().add(inner_vec.len())));
            }

            ::core::task::Poll::Ready(res)
        } else {
            debug_assert!(inner.waker.is_none());
            inner.waker.replace(cx.waker().clone());
            ::core::task::Poll::Pending
        }
    }
}

pub struct Rec<M: Memo<Args, Output> + 'static, Args, Output, T0, F0> {
    f: T0,
    arg: ::core::option::Option<Args>,
    popper_inner_stack: ::std::vec::Vec<PopperInner<Args, Output>>,
    // NOTE: Rust generated futures need actual pinning.
    future_stack: ::std::vec::Vec<::core::pin::Pin<Box<F0>>>,
    memo: M,
}

impl<
        M: Memo<Args, Output>,
        Args: Clone + 'static,
        Output: 'static,
        T0: Fn(*const dyn Fn(Args) -> Popper<Args, Output>, Args) -> F0,
        F0: ::core::future::Future<Output = Output>,
    > Rec<M, Args, Output, T0, F0>
{
    pub fn new(f: T0, memo: M) -> Self {
        Self {
            f,
            arg: ::core::option::Option::None,
            popper_inner_stack: ::std::vec::Vec::new(),
            future_stack: ::std::vec::Vec::new(),
            memo,
        }
    }

    #[allow(invalid_reference_casting)]
    pub unsafe fn me(self: ::core::pin::Pin<&Self>) -> Box<dyn Fn(Args) -> Popper<Args, Output>> {
        let this_arg_ptr = &self.arg as *const _;
        let this_popper_inner_stack_ptr = &self.popper_inner_stack as *const _;
        let this_memo_ptr = &self.memo as *const _;
        Box::new(move |args: Args| {
            let this_arg = &mut *(this_arg_ptr as *mut ::core::option::Option<Args>);
            let this_popper_inner_stack = &mut *(this_popper_inner_stack_ptr
                as *mut ::std::vec::Vec<PopperInner<Args, Output>>);
            let this_memo = &mut *(this_memo_ptr as *const _ as *mut M);
            let res = Memo::get_memo(this_memo, &args);
            if res.is_none() {
                debug_assert!(this_arg.is_none());
                this_arg.replace(Clone::clone(&args));
            }
            this_popper_inner_stack.push(PopperInner::<Args, Output> {
                waker: ::core::option::Option::None,
                res,
                args: ::core::option::Option::Some(Clone::clone(&args)),
            });
            Popper {
                // SAFETY: pointer to el of Vec may be moved, but pointer to Vec for getting last element can be treated as pinned.
                inner: this_popper_inner_stack as *const _,
            }
        })
    }

    #[allow(invalid_reference_casting)]
    pub unsafe fn call(
        self: ::core::pin::Pin<&Self>,
        me: Box<dyn Fn(Args) -> Popper<Args, Output>>,
        args: Args,
    ) -> Output {
        if let ::core::option::Option::Some(e) = Memo::get_memo(&self.memo, &args) {
            return e;
        }

        let me_ptr = &*me as *const _;
        let root_future = (self.f)(me_ptr, Clone::clone(&args));

        let this_future_stack = &mut *(&self.future_stack as *const _
            as *mut ::std::vec::Vec<::core::pin::Pin<Box<F0>>>);
        this_future_stack.push(Box::pin(root_future));

        let this_arg = &mut *(&self.arg as *const _ as *mut ::core::option::Option<Args>);
        let this_popper_inner_stack = &mut *(&self.popper_inner_stack as *const _
            as *mut ::std::vec::Vec<PopperInner<Args, Output>>);

        while let ::core::option::Option::Some(top) = this_future_stack.last() {
            // We know it'll be always waken in next loop.
            let waker = new_waker_do_nothing();
            let cx = &mut ::core::task::Context::from_waker(&waker);

            if let ::core::task::Poll::Ready(r) =
                { &mut *(top as *const _ as *mut ::core::pin::Pin<Box<F0>>) }
                    .as_mut()
                    .poll(cx)
            {
                debug_assert_eq!(this_future_stack.len(), this_popper_inner_stack.len() + 1);

                // pop unchecked
                debug_assert!(!this_future_stack.is_empty());
                this_future_stack.set_len(this_future_stack.len() - 1);
                drop(::core::ptr::read(
                    this_future_stack.as_ptr().add(this_future_stack.len()),
                ));

                if this_popper_inner_stack.is_empty() {
                    debug_assert!(this_future_stack.is_empty());
                    let this_memo = &mut *(&self.memo as *const _ as *mut M);
                    Memo::insert_memo(this_memo, Clone::clone(&args), &r);
                    return r;
                } else {
                    let top = {
                        &mut *(this_popper_inner_stack.last().unwrap_unchecked() as *const _
                            as *mut PopperInner<Args, Output>)
                    };

                    let this_memo = &mut *(&self.memo as *const _ as *mut M);
                    Memo::insert_memo(this_memo, top.args.take().unwrap_unchecked(), &r);
                    top.res = ::core::option::Option::Some(r);
                    // It's waking Rust generated Futures (F0).
                    debug_assert!(top.waker.is_some());
                    top.waker.take().unwrap_unchecked().wake();
                }

                continue;
            }
            debug_assert_eq!(this_future_stack.len(), this_popper_inner_stack.len());
            let arg = this_arg.take().unwrap_unchecked();

            let future = (self.f)(me_ptr, arg);
            this_future_stack.push(Box::pin(future));
        }
        if cfg!(debug_assertions) {
            unreachable!();
        }
        ::core::hint::unreachable_unchecked()
    }
}

/// Important difference to tuple is there is no Copy trait on this even if T is so.
pub struct ForceMover<T>(pub T);

pub trait Memo<Args, Output> {
    fn get_memo(&self, args: &Args) -> ::core::option::Option<Output>;
    fn insert_memo(&mut self, args: Args, output: &Output);
}

impl<Args: ::std::cmp::Eq + ::std::hash::Hash, Output: Clone> Memo<Args, Output>
    for ::std::collections::HashMap<Args, Output>
{
    fn get_memo(&self, args: &Args) -> ::core::option::Option<Output> {
        self.get(args).cloned()
    }
    fn insert_memo(&mut self, args: Args, output: &Output) {
        self.insert(args, output.clone());
    }
}

impl<Args: ::std::cmp::Ord, Output: Clone> Memo<Args, Output>
    for ::std::collections::BTreeMap<Args, Output>
{
    fn get_memo(&self, args: &Args) -> ::core::option::Option<Output> {
        self.get(args).cloned()
    }
    fn insert_memo(&mut self, args: Args, output: &Output) {
        self.insert(args, output.clone());
    }
}

impl<Args: ::std::cmp::Eq + ::std::hash::Hash> Memo<Args, ()>
    for ::std::collections::HashSet<Args>
{
    fn get_memo(&self, args: &Args) -> ::core::option::Option<()> {
        if self.contains(args) {
            ::core::option::Option::Some(())
        } else {
            ::core::option::Option::None
        }
    }
    fn insert_memo(&mut self, args: Args, _output: &()) {
        self.insert(args);
    }
}

impl<Args: ::std::cmp::Ord> Memo<Args, ()> for ::std::collections::BTreeSet<Args> {
    fn get_memo(&self, args: &Args) -> ::core::option::Option<()> {
        if self.contains(args) {
            ::core::option::Option::Some(())
        } else {
            ::core::option::Option::None
        }
    }
    fn insert_memo(&mut self, args: Args, _output: &()) {
        self.insert(args);
    }
}

trait SerializeAsUsize {
    fn serialize_as_usize(self) -> usize;
}

impl SerializeAsUsize for usize {
    fn serialize_as_usize(self) -> usize {
        self
    }
}

impl SerializeAsUsize for u8 {
    fn serialize_as_usize(self) -> usize {
        self as usize
    }
}

impl SerializeAsUsize for u16 {
    fn serialize_as_usize(self) -> usize {
        self as usize
    }
}

impl SerializeAsUsize for u32 {
    fn serialize_as_usize(self) -> usize {
        self as usize
    }
}

impl SerializeAsUsize for u64 {
    fn serialize_as_usize(self) -> usize {
        self as usize
    }
}

impl SerializeAsUsize for u128 {
    fn serialize_as_usize(self) -> usize {
        self as usize
    }
}

impl SerializeAsUsize for bool {
    fn serialize_as_usize(self) -> usize {
        self as usize
    }
}

impl<Args: SerializeAsUsize + Copy, Output: Clone> Memo<Args, Output>
    for ::std::vec::Vec<::core::option::Option<Output>>
{
    fn get_memo(&self, args: &Args) -> ::core::option::Option<Output> {
        match self.get(SerializeAsUsize::serialize_as_usize(*args)) {
            ::core::option::Option::Some(::core::option::Option::Some(inner)) => {
                ::core::option::Option::Some(inner.clone())
            }
            _ => ::core::option::Option::None,
        }
    }
    fn insert_memo(&mut self, args: Args, output: &Output) {
        let index = SerializeAsUsize::serialize_as_usize(args);
        while self.len() <= index {
            self.push(::core::option::Option::None);
        }
        self[index] = ::core::option::Option::Some(output.clone());
    }
}

pub struct NoMemo {}

impl<Args, Output> Memo<Args, Output> for NoMemo {
    fn get_memo(&self, _args: &Args) -> ::core::option::Option<Output> {
        ::core::option::Option::None
    }
    fn insert_memo(&mut self, _args: Args, _output: &Output) {}
}

impl<Args, Output, T: Memo<Args, Output>> Memo<Args, Output>
    for ::std::rc::Rc<::core::cell::RefCell<T>>
{
    fn get_memo(&self, args: &Args) -> Option<Output> {
        self.borrow_mut().get_memo(args)
    }
    fn insert_memo(&mut self, args: Args, output: &Output) {
        self.borrow_mut().insert_memo(args, output)
    }
}
