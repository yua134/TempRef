//! Multi thread version which used `Mutex` of TempRef. This module requires std.

extern crate std;

use core::cell::UnsafeCell;
use core::fmt::Debug;
use std::sync::{Mutex, MutexGuard, PoisonError, TryLockError};

/// A mutable reference from `Temp<T, F>`.
/// When it is dropped, it calls the reset function.
pub struct TempRef<'a, T: Send, F: FnMut(&mut T) + Send> {
    re: MutexGuard<'a, T>,
    reset: &'a mut F,
}
impl<'a, T: Send, F: FnMut(&mut T) + Send> TempRef<'a, T, F> {
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

    /// Invokes the reset function on the internal value.
    pub fn reset(&mut self) {
        (self.reset)(&mut self.re)
    }
}
impl<'a, T: Send, F: FnMut(&mut T) + Send> core::ops::Deref for TempRef<'a, T, F> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.re
    }
}
impl<'a, T: Send, F: FnMut(&mut T) + Send> core::ops::DerefMut for TempRef<'a, T, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.re
    }
}
impl<'a, T: Send, F: FnMut(&mut T) + Send> Drop for TempRef<'a, T, F> {
    fn drop(&mut self) {
        (self.reset)(&mut self.re);
    }
}
impl<'a, T: Debug + Send, F: FnMut(&mut T) + Send> Debug for TempRef<'a, T, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TempRef").field("value", &self.re).finish()
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
///
/// # Examples
/// ```
/// use tempref::mutex::Temp;
///
/// let data = vec![1;128];
/// let workspace = Temp::new(data, |d| {d.fill(0);});
///
/// assert_eq!(*workspace.lock().unwrap(), vec![1;128]);
/// // Note: The reset function is called here because MutexLock is mutable reference.
/// assert_eq!(*workspace.lock().unwrap(), vec![0;128]);
///
/// {
///     let mut guard = workspace.lock().unwrap();
///     guard.fill(1);
///     assert_eq!(*guard, vec![1;128]);
/// }
/// assert_eq!(*workspace.lock().unwrap(), vec![0;128]);
/// ```
pub struct Temp<T: Send, F: FnMut(&mut T) + Send> {
    value: Mutex<T>,
    reset: UnsafeCell<F>,
}
impl<T: Send, F: FnMut(&mut T) + Send> Temp<T, F> {
    /// A constructor of Temp<T, F>.
    pub const fn new(value: T, reset: F) -> Self {
        Temp {
            value: Mutex::new(value),
            reset: UnsafeCell::new(reset),
        }
    }
    /// A constructor of Temp<T, F>.
    ///
    /// Unlike [`Self::new`], this constructor immediately applies the given `reset`
    /// function to the initial `value` before storing it.
    pub fn new_with(mut value: T, mut reset: F) -> Self {
        reset(&mut value);
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
    /// Invokes the reset function on the internal value.
    ///
    /// This method acquires a blocking lock on the internal `Mutex<T>`.
    /// If the lock is poisoned due to a panic in another thread, it returns a `PoisonError`.
    pub fn reset<'a>(&'a self) -> Result<(), PoisonError<MutexGuard<'a, T>>> {
        unsafe { (*self.reset.get())(&mut *self.value.lock()?) }
        Ok(())
    }
    /// Attempts to invoke the reset function on the internal value.
    ///
    /// This method tries to acquire a non-blocking lock on the internal `Mutex<T>`.
    /// If the lock is already held or poisoned, it returns a `TryLockError`.
    pub fn try_reset<'a>(&'a self) -> Result<(), TryLockError<MutexGuard<'a, T>>> {
        unsafe { (*self.reset.get())(&mut *self.value.try_lock()?) }
        Ok(())
    }
}
impl<T: Default + Send, F: FnMut(&mut T) + Send> Temp<T, F> {
    /// Creates a new `Temp<T, F>` using `T::default()` as the initial value.
    pub fn new_default(reset: F) -> Self {
        Temp {
            value: Mutex::new(T::default()),
            reset: UnsafeCell::new(reset),
        }
    }

    /// Creates a new `Temp<T, F>` using `T::default()` as the initial value,
    /// and immediately applies the given `reset` function to it.
    ///
    /// This is similar to [`Self::new_default`], but the `reset` function is called once
    /// during initialization.
    pub fn new_default_with(mut reset: F) -> Self {
        let mut default = T::default();
        reset(&mut default);
        Temp {
            value: Mutex::new(default),
            reset: UnsafeCell::new(reset),
        }
    }
}
unsafe impl<T: Send, F: FnMut(&mut T) + Send> Send for Temp<T, F> {}
unsafe impl<T: Send, F: FnMut(&mut T) + Send> Sync for Temp<T, F> {}
impl<T: Debug + Send, F: FnMut(&mut T) + Send> Debug for Temp<T, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Temp").field("value", &self.value).finish()
    }
}
