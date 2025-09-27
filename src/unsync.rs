//! Single thread version of TempRef. This module doesn't require std.

use core::cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut, UnsafeCell};

/// A mutable reference wrapper from [`Temp<T, F>`].
///
/// When dropped, it automatically calls the reset function on the underlying value.
/// This ensures that temporary mutations never leave the value in an inconsistent state.
#[derive(Debug)]
pub struct TempRef<'a, T, F: FnMut(&mut T)> {
    re: RefMut<'a, T>,
    reset: &'a mut F,
}
impl<'a, T, F: FnMut(&mut T)> TempRef<'a, T, F> {
    fn new(temp: &'a Temp<T, F>) -> Self {
        TempRef {
            re: temp.value.borrow_mut(),
            reset: unsafe { &mut *temp.reset.get() },
        }
    }
    fn try_new(temp: &'a Temp<T, F>) -> Result<Self, BorrowMutError> {
        Ok(TempRef {
            re: temp.value.try_borrow_mut()?,
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

/// A value wrapper that ensures its mutable reference is always reset when dropped.
///
/// `Temp<T, F>` holds a value of type `T` inside a `RefCell`, together with a reset
/// function `F: FnMut(&mut T)`. Every time a mutable borrow is created via [`Self::borrow_mut`],
/// the returned [`TempRef`] will call the reset function when dropped.
///
/// This can be useful for values that must always be returned to a default or
/// safe state after temporary modification.
#[derive(Debug)]
pub struct Temp<T, F: FnMut(&mut T)> {
    value: RefCell<T>,
    reset: UnsafeCell<F>,
}
impl<T, F: FnMut(&mut T)> Temp<T, F> {
    /// A constructor of Temp<T, F>.
    pub fn new(value: T, reset: F) -> Self {
        Temp {
            value: RefCell::new(value),
            reset: UnsafeCell::new(reset),
        }
    }
    /// Immutably borrows the wrapped value.
    /// The borrow lasts until the returned Ref exits scope. Multiple immutable borrows can be taken out at the same time
    pub fn borrow<'a>(&'a self) -> Ref<'a, T> {
        self.value.borrow()
    }
    /// Mutably borrows the wrapped value as `TempRef`.
    /// The value cannot be borrowed while this borrow is active.
    /// Automatically resets itself when dropped.
    pub fn borrow_mut<'a>(&'a self) -> TempRef<'a, T, F> {
        TempRef::new(self)
    }
    /// A safer function; `self.borrow()`.
    pub fn try_borrow<'a>(&'a self) -> Result<Ref<'a, T>, BorrowError> {
        self.value.try_borrow()
    }
    /// A safer function; `self.borrow_mut()`.
    pub fn try_borrow_mut<'a>(&'a self) -> Result<TempRef<'a, T, F>, BorrowMutError> {
        TempRef::try_new(self)
    }
    /// Consumes the `Temp`, returning the wrapped value.
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
    /// Replaces the wrapped value with a new one, returning the old value, without deinitializing either one.
    pub fn replace(&self, value: T) -> T {
        self.value.replace(value)
    }
    /// Replaces the wrapped value with a new one computed from f, returning the old value, without deinitializing either one.
    pub fn replace_with<C: FnOnce(&mut T) -> T>(&self, f: C) -> T {
        self.value.replace_with(f)
    }
    /// Swaps the wrapped value of self with the wrapped value of other, without deinitializing either one.
    pub fn swap(&self, other: T) {
        self.value.swap(&RefCell::new(other));
    }
}
