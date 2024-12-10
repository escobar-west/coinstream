use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering::*},
};

pub struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // Safety: The very existence of this Guard
        // guarantees we've exclusively locked the lock.
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // Safety: The very existence of this Guard
        // guarantees we've exclusively locked the lock.
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.is_locked.store(false, Release);
    }
}

pub struct SpinLock<T> {
    is_locked: AtomicBool,
    value: UnsafeCell<T>,
}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            is_locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> Guard<T> {
        while self.is_locked.swap(true, Acquire) {
            std::hint::spin_loop();
        }
        Guard { lock: self }
    }
}

unsafe impl<T> Sync for SpinLock<T> where T: Send {}
