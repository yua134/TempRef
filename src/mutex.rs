//! Multi thread version which used `Mutex` of TempRef. This module requires std.

extern crate std;

use core::cell::UnsafeCell;
use std::sync::{Mutex, MutexGuard, PoisonError, TryLockError};

/// A mutable reference from `Temp<T, F>`.
/// When it is dropped, it calls the reset function.
#[derive(Debug)]
pub struct TempRef<'a, T, F: FnMut(&mut T)> {
    re: MutexGuard<'a, T>,
    reset: &'a mut F,
}
impl<'a, T: Send, F: FnMut(&mut T) + Sync> TempRef<'a, T, F> {
    fn new(temp: &'a Temp<T, F>) -> Result<Self, PoisonError<MutexGuard<'a, T>>> {
        Ok(TempRef {
            re: temp.value.lock()?,
            reset: unsafe { &mut *temp.reset.get() },
        })
    }
    fn try_new(temp: &'a Temp<T, F>) -> Result<Self, TryLockError<MutexGuard<'a, T>>> {
        Ok(TempRef {
            re: temp.value.try_lock()?,
            reset: unsafe { &mut *temp.reset.get() },
        })
    }
}
impl<'a, T, F: FnMut(&mut T)> core::ops::Deref for TempRef<'a, T, F> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.re
    }
}
impl<'a, T, F: FnMut(&mut T)> core::ops::DerefMut for TempRef<'a, T, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.re
    }
}
impl<'a, T, F: FnMut(&mut T)> Drop for TempRef<'a, T, F> {
    fn drop(&mut self) {
        (self.reset)(&mut self.re);
    }
}

/// A value protected by a `Mutex` that ensures its mutable reference is always reset when dropped.
///
/// `Temp<T, F>` holds a value of type `T` inside a `Mutex`, together with a reset
/// function `F: FnMut(&mut T)`. Every time a mutable borrow is created via [`Self::lock`]
/// or [`Self::try_lock`], the returned [`TempRef`] will call the reset function when dropped.
///
/// This guarantees that temporary mutations never leave the value in an
/// inconsistent state, even in multithreaded contexts.
#[derive(Debug)]
pub struct Temp<T: Send, F: FnMut(&mut T) + Sync> {
    value: Mutex<T>,
    reset: UnsafeCell<F>,
}
impl<T: Send, F: FnMut(&mut T) + Sync> Temp<T, F> {
    /// A constructor of Temp<T, F>.
    pub fn new(value: T, reset: F) -> Self {
        Temp {
            value: Mutex::new(value),
            reset: UnsafeCell::new(reset),
        }
    }
    /// Creates `TempRef`.
    /// Automatically resets itself when dropped.
    /// Acquires a mutex, blocking the current thread until it is able to do so.
    pub fn lock<'a>(&'a self) -> Result<TempRef<'a, T, F>, PoisonError<MutexGuard<'a, T>>> {
        TempRef::new(self)
    }
    /// Attempts to acquire this lock.
    /// If the lock could not be acquired at this time, then Err is returned. Otherwise, TempRef is returned.
    pub fn try_lock<'a>(&'a self) -> Result<TempRef<'a, T, F>, TryLockError<MutexGuard<'a, T>>> {
        TempRef::try_new(self)
    }
    /// Consumes the Temp, returning the wrapped value.
    pub fn into_inner(self) -> Result<T, PoisonError<T>> {
        self.value.into_inner()
    }
    /// Clear the poisoned state from a mutex.
    pub fn clear_poison(&self) {
        self.value.clear_poison();
    }
    /// Determines whether the mutex is poisoned.
    pub fn is_poisoned(&self) -> bool {
        self.value.is_poisoned()
    }
}
unsafe impl<T: Send, F: FnMut(&mut T) + Sync> Send for Temp<T, F> {}
unsafe impl<T: Send, F: FnMut(&mut T) + Sync> Sync for Temp<T, F> {}
