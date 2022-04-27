use core::future::Future;
use core::pin::Pin;
use core::task::{Poll, Context};
use core::task::{RawWaker, RawWakerVTable, Waker};

use alloc::collections::VecDeque;

#[macro_use]
use alloc::vec;
use alloc::vec::Vec;
use alloc::boxed::Box;

enum State {
    Halted,
    Running,
}

struct Fib {
    state: State,
}

impl Fib {
    fn waiter<'a>(&'a mut self) -> Waiter<'a> {
        Waiter { fib: self }
    }
}

struct Waiter<'a> {
    fib: &'a mut Fib,
}

impl<'a> Future for Waiter<'a> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        match self.fib.state {
            State::Halted => {
                self.fib.state = State::Running;
                Poll::Ready(())
            }
            State::Running => {
                self.fib.state = State::Halted;
                Poll::Pending
            }
        }
    }
}



struct Executor {
    fibs: VecDeque<Pin<Box<dyn Future<Output=()>>>>,
}

impl Executor {
    fn new() -> Self {
        Executor {
            fibs: VecDeque::new(),
        }
    }

    fn push<C, F>(&mut self, closure: C)
    where
        F: Future<Output=()> + 'static,
        C: FnOnce(Fib) -> F,
    {
        let fib = Fib { state: State::Running };
        self.fibs.push_back(Box::pin(closure(fib)));
    }

    fn run(&mut self) {
        let waker = create();
        let mut context = Context::from_waker(&waker);

        while let Some(mut fib) = self.fibs.pop_front() {
            match fib.as_mut().poll(&mut context) {
                Poll::Pending => {
                    self.fibs.push_back(fib);
                },
                Poll::Ready(()) => {},
            }
        }
    }
}




pub fn create() -> Waker {
    // Safety: The waker points to a vtable with functions that do nothing. Doing
    // nothing is memory-safe.
    unsafe { Waker::from_raw(RAW_WAKER) }
}

const RAW_WAKER: RawWaker = RawWaker::new(core::ptr::null(), &VTABLE);
const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

unsafe fn clone(_: *const ()) -> RawWaker { RAW_WAKER }
unsafe fn wake(_: *const ()) { }
unsafe fn wake_by_ref(_: *const ()) { }
unsafe fn drop(_: *const ()) { }



pub fn stackless_coroutine_test() {
    let mut exec = Executor::new();

    for instance in 1..=3 {
        exec.push(move |mut fib| async move {
            println!("{} A", instance);
            fib.waiter().await;
            println!("{} B", instance);
            fib.waiter().await;
            println!("{} C", instance);
            fib.waiter().await;
            println!("{} D", instance);
        });
    }

    println!("Running");
    exec.run();
    println!("Done");
}