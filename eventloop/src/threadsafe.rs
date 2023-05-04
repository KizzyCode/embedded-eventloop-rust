//! Implements an threadsafe wrapper

use crate::runtime;
use core::{
    cell::UnsafeCell,
    fmt::{self, Debug, Formatter},
};

/// A threadsafe cell
pub struct ThreadSafeCell<T> {
    /// The wrapped value
    inner: UnsafeCell<T>,
}
impl<T> ThreadSafeCell<T> {
    /// Creates a new threadsafe Cell
    pub const fn new(value: T) -> Self {
        Self { inner: UnsafeCell::new(value) }
    }

    /// Provides scoped access to the underlying value
    pub fn scope<F, FR>(&self, scope: F) -> FR
    where
        F: FnOnce(&mut T) -> FR,
    {
        // Create mutable slots to transfer state to/from the closure
        let mut scope = Some(scope);
        let mut result: Option<FR> = None;

        // Create the implementation
        let mut impl_ = || {
            // Get the value
            let value_ptr = self.inner.get();
            let value = unsafe { value_ptr.as_mut() }.expect("unexpected NULL pointer inside cell");

            // Take the scope
            let scope = scope.take().expect("missing scope function");
            result = Some(scope(value));
        };

        // Run the implementation in a threadsafe context and return the result
        unsafe { runtime::_eventloop_0_1_threadsafe(&mut impl_) };
        result.expect("implementation scope did not set result value")
    }
}
impl<T> Debug for ThreadSafeCell<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.scope(|value| f.debug_struct("ThreadSafe").field("value", value).finish())
    }
}
unsafe impl<T> Sync for ThreadSafeCell<T>
where
    T: Send,
{
    // Marker trait, no members to implement
}
