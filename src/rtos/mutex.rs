use core::{
    cell::UnsafeCell,
    fmt,
    fmt::{Debug, Display, Formatter},
    ops::{Deref, DerefMut},
};

use crate::{bindings, error::*};

use super::TIMEOUT_MAX;

/// Represents an object which is protected by a FreeRTOS mutex.
pub struct Mutex<T: ?Sized> {
    mutex: bindings::mutex_t,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

unsafe impl<T: ?Sized + Sync> Sync for Mutex<T> {}

/// Provides a constant-period looping construct.
impl<T> Mutex<T> {
    #[inline]
    /// Creates a new mutex which wraps the given object. Panics on failure; see
    /// [`Mutex::try_new()`].
    pub fn new(data: T) -> Self {
        Self::try_new(data).unwrap_or_else(|err| panic!("failed to create mutex: {:?}", err))
    }

    /// Creates a new mutex which wraps the given object.
    pub fn try_new(data: T) -> Result<Self, Error> {
        let mutex = unsafe { bindings::mutex_create() };
        if mutex.is_null() {
            Err(from_errno())
        } else {
            Ok(Self {
                data: UnsafeCell::new(data),
                mutex,
            })
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    #[inline]
    /// Obtains a [`MutexGuard`] giving access to the object protected by the
    /// mutex. Blocks until access can be obtained. Panics on failure; see
    /// [`Mutex::try_lock()`].
    pub fn lock(&'_ self) -> MutexGuard<'_, T> {
        self.try_lock()
            .unwrap_or_else(|err| panic!("Failed to lock mutex: {:?}", err))
    }

    #[inline]
    /// Obtains a [`MutexGuard`] giving access to the object protected by the
    /// mutex. Blocks until access can be obtained.
    pub fn try_lock(&'_ self) -> Result<MutexGuard<'_, T>, Error> {
        if unsafe { bindings::mutex_take(self.mutex, TIMEOUT_MAX) } {
            Ok(MutexGuard(self))
        } else {
            Err(from_errno())
        }
    }

    #[inline]
    /// Obtains a [`MutexGuard`] giving access to the object protected by the
    /// mutex, if it is available immediately. Does not block.
    pub fn poll(&'_ self) -> Option<MutexGuard<'_, T>> {
        if unsafe { bindings::mutex_take(self.mutex, 0) } {
            Some(MutexGuard(self))
        } else {
            None
        }
    }
}

impl<T: ?Sized> Drop for Mutex<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { bindings::mutex_delete(self.mutex) }
    }
}

impl<T: ?Sized + Debug> Debug for Mutex<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.poll() {
            Some(guard) => f.debug_struct("Mutex").field("data", &&*guard).finish(),
            None => {
                struct LockedPlaceholder;
                impl Debug for LockedPlaceholder {
                    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                        f.write_str("<locked>")
                    }
                }

                f.debug_struct("Mutex")
                    .field("data", &LockedPlaceholder)
                    .finish()
            }
        }
    }
}

impl<T: ?Sized + Default> Default for Mutex<T> {
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> From<T> for Mutex<T> {
    #[inline]
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

/// Provides exclusive access to an object controlled by a [`Mutex`] via the
/// RAII pattern.
pub struct MutexGuard<'a, T: ?Sized>(&'a Mutex<T>);

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.data.get() }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        if !unsafe { bindings::mutex_give(self.0.mutex) } {
            panic!("failed to return mutex: {:?}", from_errno());
        }
    }
}

impl<T: ?Sized + Debug> Debug for MutexGuard<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized + Display> Display for MutexGuard<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized> !Send for MutexGuard<'_, T> {}

unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}
