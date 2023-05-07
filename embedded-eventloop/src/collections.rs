//! A stack-allocated ring buffer implementation

use core::{array::IntoIter, iter::Flatten};

/// A push-only stack-allocated stack for `Copy`-types
#[derive(Debug, Clone, Copy)]
pub struct Stack<T, const SIZE: usize>
where
    T: Copy,
{
    /// The underlying elements
    elements: [Option<T>; SIZE],
    /// The amount of elements
    len: usize,
}
impl<T, const SIZE: usize> Stack<T, SIZE>
where
    T: Copy,
{
    /// The default value for non-copy const-time initialization
    const INIT: Option<T> = None;

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
        self.elements[self.len] = Some(value);
        self.len += 1;
        Ok(())
    }
}
impl<T, const SIZE: usize> IntoIterator for Stack<T, SIZE>
where
    T: Copy,
{
    type Item = T;
    type IntoIter = Flatten<IntoIter<Option<T>, SIZE>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter().flatten()
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
