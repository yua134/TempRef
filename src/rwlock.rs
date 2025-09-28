//! Multi thread version which used `RwLock` of TempRef. This module requires std.

extern crate std;

use core::cell::UnsafeCell;
use core::fmt::Debug;
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};

/// A mutable reference wrapper from [`Temp<T, F>`].
///
/// When dropped, it automatically calls the reset function on the underlying value.
/// This ensures that temporary mutations never leave the value in an inconsistent state.
pub struct TempRef<'a, T: Send, F: FnMut(&mut T) + Sync> {
    re: RwLockWriteGuard<'a, T>,
    reset: &'a mut F,
}
impl<'a, T: Send, F: FnMut(&mut T) + Sync> TempRef<'a, T, F> {
    fn new(temp: &'a Temp<T, F>) -> Result<Self, PoisonError<RwLockWriteGuard<'a, T>>> {
        Ok(TempRef {
            re: temp.value.write()?,
            reset: unsafe { &mut *temp.reset.get() },
        })
    }
    fn try_new(temp: &'a Temp<T, F>) -> Result<Self, TryLockError<RwLockWriteGuard<'a, T>>> {
        Ok(TempRef {
            re: temp.value.try_write()?,
            reset: unsafe { &mut *temp.reset.get() },
        })
    }

    /// Invokes the reset function on the internal value.
    pub fn reset(&mut self) {
        (self.reset)(&mut self.re);
    }
}
impl<'a, T: Send, F: FnMut(&mut T) + Sync> core::ops::Deref for TempRef<'a, T, F> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.re
    }
}
impl<'a, T: Send, F: FnMut(&mut T) + Sync> core::ops::DerefMut for TempRef<'a, T, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.re
    }
}
impl<'a, T: Send, F: FnMut(&mut T) + Sync> Drop for TempRef<'a, T, F> {
    fn drop(&mut self) {
        (self.reset)(&mut self.re);
    }
}
impl<'a, T: Send + Debug, F: FnMut(&mut T) + Sync> Debug for TempRef<'a, T, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TempRef").field("value", &self.re).finish()
    }
}

/// A value protected by `RwLock` that ensures its mutable reference is always reset when dropped.
///
/// `Temp<T, F>` holds a value of type `T` inside an `RwLock`, together with a reset
/// function `F: Mut(&mut T)`. Every time a mutable borrow is created via [`Self::write`],
/// the returned [`TempRef`] will call the reset function when dropped.
///
/// This guarantees that temporary mutations in a multithreaded context
/// never leave the value in an inconsistent state.
///
/// # Examples
/// ```
/// use tempref::rwlock::Temp;
///
/// let data = vec![0;128];
/// let workspace = Temp::new(data, |d| {d.fill(0);});
///
/// assert_eq!(*workspace.read().unwrap(), vec![0;128]);
///
/// {
///     let mut guard = workspace.write().unwrap();
///     guard.fill(1);
///     assert_eq!(*guard, vec![1;128]);
/// }
/// assert_eq!(*workspace.read().unwrap(), vec![0;128]);
/// ```
pub struct Temp<T: Send, F: FnMut(&mut T) + Sync> {
    value: RwLock<T>,
    reset: UnsafeCell<F>,
}
impl<T: Send, F: FnMut(&mut T) + Sync> Temp<T, F> {
    /// A constructor of Temp<T, F>.
    pub const fn new(value: T, reset: F) -> Self {
        Temp {
            value: RwLock::new(value),
            reset: UnsafeCell::new(reset),
        }
    }
    /// Locks this Temp with shared read access, blocking the current thread until it can be acquired.
    pub fn read<'a>(
        &'a self,
    ) -> Result<RwLockReadGuard<'a, T>, PoisonError<RwLockReadGuard<'a, T>>> {
        self.value.read()
    }
    /// Acquires an exclusive write lock on this `Temp`, blocking the current thread until the lock is available.
    /// The returned `TempRef` automatically resets itself when dropped.
    pub fn write<'a>(&'a self) -> Result<TempRef<'a, T, F>, PoisonError<RwLockWriteGuard<'a, T>>> {
        TempRef::new(self)
    }
    /// Attempts to acquire this Temp with shared read access.
    /// If the access could not be granted at this time, then Err is returned. Otherwise, an RAII guard is returned which will release the shared access when it is dropped.
    pub fn try_read<'a>(
        &'a self,
    ) -> Result<RwLockReadGuard<'a, T>, TryLockError<RwLockReadGuard<'a, T>>> {
        self.value.try_read()
    }
    /// Attempts to lock this Temp with exclusive write access.
    /// If the lock could not be acquired at this time, then Err is returned. Otherwise, TempRef is returned which will release the lock when it is dropped.
    /// Automatically resets itself when dropped.
    pub fn try_write<'a>(
        &'a self,
    ) -> Result<TempRef<'a, T, F>, TryLockError<RwLockWriteGuard<'a, T>>> {
        TempRef::try_new(self)
    }
    /// Consumes this Temp, returning the underlying data.
    pub fn into_inner(self) -> Result<T, PoisonError<T>> {
        self.value.into_inner()
    }
    /// Clear the poisoned state from a lock.
    pub fn clear_poison(&self) {
        self.value.clear_poison();
    }
    /// Determines whether the lock is poisoned.
    pub fn is_poisoned(&self) -> bool {
        self.value.is_poisoned()
    }
    /// Invokes the reset function on the internal value.
    ///
    /// This method acquires a blocking write lock on the internal value.
    /// If the lock is poisoned, it returns a `PoisonError`.
    pub fn reset<'a>(&'a self) -> Result<(), PoisonError<RwLockWriteGuard<'a, T>>> {
        unsafe { (*self.reset.get())(&mut *self.value.write()?) }
        Ok(())
    }
    /// Attempts to invoke the reset function on the internal value.
    ///
    /// This method tries to acquire a non-blocking write lock on the internal value.
    /// If the lock cannot be immediately acquired, it returns a `TryLockError`.
    pub fn try_reset<'a>(&'a self) -> Result<(), TryLockError<RwLockWriteGuard<'a, T>>> {
        unsafe { (*self.reset.get())(&mut *self.value.try_write()?) }
        Ok(())
    }
}
unsafe impl<T: Send, F: FnMut(&mut T) + Sync> Send for Temp<T, F> {}
unsafe impl<T: Send, F: FnMut(&mut T) + Sync> Sync for Temp<T, F> {}
impl<T: Debug + Send, F: FnMut(&mut T) + Sync> Debug for Temp<T, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Temp").field("value", &self.value).finish()
    }
}
