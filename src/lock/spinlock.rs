use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

pub struct SpinLock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLock<T> {}
unsafe impl<T: Send> Send for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(inner),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<T> {
        #[allow(clippy::never_loop)]
        while self
            .lock
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
            .is_err()
        {
            // unsafe { crate::port::Port::new(0x3F8).write(b'A') };
            panic!("actually locking: {:?}", core::any::type_name::<T>());
        }
        debug_assert!(self.lock.load(Ordering::Acquire));
        SpinLockGuard::new(&self.lock, self.data.get())
    }
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a AtomicBool,
    data: *mut T,
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.data.as_ref().unwrap() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.data.as_mut().unwrap() }
    }
}

impl<'a, T> SpinLockGuard<'a, T> {
    pub fn new(lock: &'a AtomicBool, data: *mut T) -> Self {
        Self { lock, data }
    }
}
