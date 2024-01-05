//! Uniprocessor interior mutability primitives
use core::cell::{RefCell, RefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// Wrap a static data structure inside it so that we are
/// able to access it without any `unsafe`.
///
/// We should only use it in uniprocessor.
///
/// In order to get mutable reference of inner data, call
/// `exclusive_access`.
pub struct UPSafeCell<T> {
    /// inner data
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    /// User is responsible to guarantee that inner struct is only used in
    /// uniprocessor.
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    /// Exclusive access inner data in UPSafeCell. Panic if the data has been borrowed.
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

/// The sync primitive used by easy-fs.
pub struct RawExclusiveLock(AtomicBool);

unsafe impl lock_api::RawMutex for RawExclusiveLock {
    const INIT: Self = Self(AtomicBool::new(false));
    type GuardMarker = lock_api::GuardNoSend;
    fn lock(&self) {
        assert_eq!(self.0.load(Ordering::Relaxed), false);
        self.0.store(true, Ordering::Relaxed);
    }
    fn try_lock(&self) -> bool {
        self.0
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
    }
    unsafe fn unlock(&self) {
        self.0.store(false, Ordering::Relaxed);
    }
}
