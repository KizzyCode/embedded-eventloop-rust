//! A threadsafe cell type

use crate::runtime;
use core::{
    cell::UnsafeCell,
    fmt::{self, Debug, Formatter},
};

/// A threadsafe cell
#[repr(transparent)]
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

        // Create the caller
        let mut call_scope = || {
            // Get the value
            let value_ptr = self.inner.get();
            let value = unsafe { value_ptr.as_mut() }.expect("unexpected NULL pointer inside cell");

            // Take the scope
            let scope = scope.take().expect("missing scope function");
            result = Some(scope(value));
        };

        // Run the implementation in a threadsafe context and return the result
        unsafe { runtime::_runtime_threadsafe_NfpNM21J(&mut call_scope) };
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
