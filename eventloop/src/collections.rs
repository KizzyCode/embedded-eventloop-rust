//! A stack-allocated ring buffer implementation

use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    slice,
};

/// A push-only stack-allocated stack
#[derive(Debug)]
pub struct Stack<T, const SIZE: usize> {
    /// The underlying elements
    elements: [MaybeUninit<T>; SIZE],
    /// The amount of elements
    len: usize,
}
impl<T, const SIZE: usize> Stack<T, SIZE> {
    /// The default value for non-copy const-time initialization
    const INIT: MaybeUninit<T> = MaybeUninit::uninit();

    /// Creates a new stack
    pub const fn new() -> Self {
        Self { elements: [Self::INIT; SIZE], len: 0 }
    }

    /// Pushes a value onto the stack
    pub fn push(&mut self, value: T) -> Result<(), T> {
        // Ensure that we have a free slot
        if self.len == SIZE {
            return Err(value);
        }

        // Insert the value
        self.elements[self.len].write(value);
        self.len += 1;
        Ok(())
    }
}
impl<T, const SIZE: usize> Drop for Stack<T, SIZE> {
    fn drop(&mut self) {
        // Drop the initialized elements
        for element in self.elements.iter_mut().take(self.len) {
            unsafe { element.assume_init_drop() };
        }
    }
}
impl<T, const SIZE: usize> Deref for Stack<T, SIZE> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // This feels dirty, but should be sound since "MaybeUninit<T> is guaranteed to have the same size, alignment, and
        // ABI as T" (https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#layout-1)
        let ptr = self.elements.as_ptr() as *const T;
        unsafe { slice::from_raw_parts(ptr, self.len) }
    }
}
impl<T, const SIZE: usize> DerefMut for Stack<T, SIZE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // This feels dirty, but should be sound since "MaybeUninit<T> is guaranteed to have the same size, alignment, and
        // ABI as T" (https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#layout-1)
        let ptr = self.elements.as_mut_ptr() as *mut T;
        unsafe { slice::from_raw_parts_mut(ptr, self.len) }
    }
}

/// A stack-allocated ring buffer
#[derive(Debug)]
pub struct RingBuf<T, const SIZE: usize> {
    /// The ring buffer
    buf: [Option<T>; SIZE],
    /// The position of the next free slot
    head: usize,
    /// The position of the next pending element
    tail: usize,
}
impl<T, const SIZE: usize> RingBuf<T, SIZE> {
    /// The default value for non-copy const-time initialization
    const INIT: Option<T> = None;

    /// Creates a new empty ring buffer
    pub const fn new() -> Self {
        Self { buf: [Self::INIT; SIZE], head: 0, tail: 0 }
    }

    /// Pushes `element` into the ring buffer
    pub fn push(&mut self, element: T) -> Result<(), T> {
        // Check if the head-slot is empty, otherwise the queue is full
        if self.buf[self.head].is_some() {
            return Err(element);
        }

        // Insert the element into the buffer
        self.buf[self.head] = Some(element);
        self.head = (self.head + 1) % SIZE;
        Ok(())
    }
    /// Pops an element from the ring buffer
    pub fn pop(&mut self) -> Option<T> {
        // Take the element
        let element = self.buf[self.tail].take()?;
        self.tail = (self.tail + 1) % SIZE;
        Some(element)
    }
}
