/*!
 * `Send`/`Sync` behaviors tailored for WASM
 */

use std::ops::{Deref, DerefMut};

/// ReadWrite lock. Sync when the `sync` feature is enabled.
pub trait ReadWriteLock<T: ?Sized> {
    /// RAII immutable reference
    type ReadGuard<'a>: 'a + Deref<Target = T>
    where
        Self: 'a;
    /// RAII mutable reference
    type WriteGuard<'a>: 'a + DerefMut<Target = T>
    where
        Self: 'a;

    /// Acquire a read-lock on the value
    fn read(&self) -> Self::ReadGuard<'_>;
    /// Acquire a write-lock on the value
    fn write(&self) -> Self::WriteGuard<'_>;

    /// Try to acquire a read-lock on the value. Allowed to fail.
    fn try_read(&self) -> Option<Self::ReadGuard<'_>> {
        Some(self.read())
    }

    /// Try to acquire a write-lock on the value. Allowed to fail.
    fn try_write(&self) -> Option<Self::WriteGuard<'_>> {
        Some(self.write())
    }

    /// Take the current value and replace it with default
    fn take(&self) -> T
    where
        T: Default,
    {
        std::mem::replace(&mut *self.write(), Default::default())
    }
}

pub use imp::{Arc, RwLock};

#[cfg(any(target_arch = "wasm32", not(feature = "sync")))]
mod imp {
    /// Read-Write Lock
    pub type RwLock<T> = RefCell<T>;
    /// Reference Counted Smart Pointer
    pub type Arc<T> = std::rc::Rc<T>;

    use super::ReadWriteLock;
    use std::cell::{Ref, RefCell, RefMut};

    impl<T: ?Sized> ReadWriteLock<T> for RefCell<T> {
        type ReadGuard<'a> = Ref<'a, T> where T: 'a;
        type WriteGuard<'a> = RefMut<'a, T> where T: 'a;

        fn read(&self) -> Self::ReadGuard<'_> {
            self.borrow()
        }

        fn write(&self) -> Self::WriteGuard<'_> {
            self.borrow_mut()
        }

        fn try_read(&self) -> Option<Self::ReadGuard<'_>> {
            self.try_borrow().ok()
        }

        fn try_write(&self) -> Option<Self::WriteGuard<'_>> {
            self.try_borrow_mut().ok()
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "sync"))]
mod imp {
    /// Read-Write Lock
    #[derive(Default, Debug)]
    #[repr(transparent)]
    pub struct RwLock<T: ?Sized>(StdRwLock<T>);

    unsafe impl<T: ?Sized + Send> Send for RwLock<T> {}
    unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}

    impl<T> RwLock<T> {
        /// Creates a new instance of an `RwLock<T>` which is unlocked.
        pub const fn new(value: T) -> Self {
            RwLock(StdRwLock::new(value))
        }
    }

    /// Reference Counted Smart Pointer
    pub type Arc<T> = std::sync::Arc<T>;

    use super::ReadWriteLock;

    #[cfg(not(feature = "parking_lot"))]
    use std::sync::{RwLock as StdRwLock, RwLockReadGuard, RwLockWriteGuard};

    #[cfg(not(feature = "parking_lot"))]
    impl<T: ?Sized> ReadWriteLock<T> for RwLock<T> {
        type ReadGuard<'a> = RwLockReadGuard<'a, T> where T: 'a;
        type WriteGuard<'a> = RwLockWriteGuard<'a, T> where T: 'a;

        fn read(&self) -> Self::ReadGuard<'_> {
            self.0.read().unwrap()
        }

        fn write(&self) -> Self::WriteGuard<'_> {
            self.0.write().unwrap()
        }

        fn try_read(&self) -> Option<Self::ReadGuard<'_>> {
            self.0.try_read().ok()
        }

        fn try_write(&self) -> Option<Self::WriteGuard<'_>> {
            self.0.try_write().ok()
        }
    }

    #[cfg(feature = "parking_lot")]
    use parking_lot::{RwLock as StdRwLock, RwLockReadGuard, RwLockWriteGuard};

    #[cfg(feature = "parking_lot")]
    impl<T: ?Sized> ReadWriteLock<T> for RwLock<T> {
        type ReadGuard<'a> = RwLockReadGuard<'a, T> where T: 'a;
        type WriteGuard<'a> = RwLockWriteGuard<'a, T> where T: 'a;

        fn read(&self) -> Self::ReadGuard<'_> {
            self.0.read()
        }

        fn write(&self) -> Self::WriteGuard<'_> {
            self.0.write()
        }

        fn try_read(&self) -> Option<Self::ReadGuard<'_>> {
            self.0.try_read()
        }

        fn try_write(&self) -> Option<Self::WriteGuard<'_>> {
            self.0.try_write()
        }
    }
}
