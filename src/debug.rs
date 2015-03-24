use std::cell::Cell;
use std::sync::atomic::{AtomicIsize, ATOMIC_ISIZE_INIT, Ordering};

thread_local!(pub static ENABLED: Cell<u32> = Cell::new(0));

#[cfg(test)]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => (
        ::debug::ENABLED.with(|slot| {
            if slot.get() != 0 {
                println!("{}", format_args!($($arg)+));
            }
        })
    )
}

#[cfg(not(test))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ( () )
}

pub struct Logger { _x: () }

impl Logger {
    pub fn new() -> Logger {
        ENABLED.with(|slot| slot.set(slot.get() + 1));
        Logger { _x: () }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        ENABLED.with(|slot| slot.set(slot.get() - 1));
    }
}
