//! An event loop

use crate::{
    boxes::{Box, CopyBox},
    collections::{RingBuf, Stack},
    runtime,
    threadsafe::ThreadSafeCell,
};
use core::{any::TypeId, mem};

/// The size of a function pointer
const FPTR_SIZE: usize = mem::size_of::<fn()>();

/// An event listener with the associated type and a type-specific caller implementation
#[derive(Debug, Clone, Copy)]
struct EventListener<const SIZE: usize> {
    /// The type ID
    pub type_id: TypeId,
    /// The boxed callback
    pub callback_box: CopyBox<FPTR_SIZE>,
    /// A type specific caller to invoke the callback
    pub caller: fn(Box<SIZE>, CopyBox<FPTR_SIZE>) -> Option<Box<SIZE>>,
}

/// An event loop
#[derive(Debug)]
pub struct EventLoop<const STACKBOX_SIZE: usize = 64, const BACKLOG_MAX: usize = 32, const LISTENERS_MAX: usize = 32> {
    /// The event buffer
    events: ThreadSafeCell<RingBuf<Box<STACKBOX_SIZE>, BACKLOG_MAX>>,
    /// The event listeners
    listeners: ThreadSafeCell<Stack<EventListener<STACKBOX_SIZE>, LISTENERS_MAX>>,
}
impl<const STACKBOX_SIZE: usize, const BACKLOG_MAX: usize, const LISTENERS_MAX: usize>
    EventLoop<STACKBOX_SIZE, BACKLOG_MAX, LISTENERS_MAX>
{
    /// Creates a new event loop
    pub const fn new() -> Self {
        let events = ThreadSafeCell::new(RingBuf::new());
        let listeners = ThreadSafeCell::new(Stack::new());
        Self { events, listeners }
    }

    /// Adds a listener to the event loop which receives all events of type `T`
    ///
    /// # Note on multiple listeners
    /// It is possible to chain multiple listeners for the same event type `T`. If the first invoked listener returns
    /// `Some(event)` again, the next listener is invoked with `event`, and so on. If at some point a listener returns
    /// `None`, the chain ends and subsequent listeners are not invoked anymore.
    pub fn listen<T>(&self, callback: fn(T) -> Option<T>) -> Result<(), fn(T) -> Option<T>>
    where
        T: 'static,
    {
        // Create the caller
        let callback_box = CopyBox::new(callback).expect("cannot box function pointer");
        let caller: fn(Box<STACKBOX_SIZE>, CopyBox<FPTR_SIZE>) -> Option<Box<STACKBOX_SIZE>> = Self::caller::<T>;
        let listener = EventListener { type_id: TypeId::of::<T>(), callback_box, caller };

        // Insert the listener
        if self.listeners.scope(|listeners| listeners.push(listener)).is_err() {
            return Err(callback);
        }
        Ok(())
    }
    /// Adds a listener to the event loop which receives all events of type `T`, and sends `event` to ensure that the
    /// listener is at least called once
    ///
    /// This method is especially useful to bootstrap periodical event sources (e.g. timers).
    ///
    /// # Note on multiple listeners
    /// It is possible to chain multiple listeners for the same event type `T`. If the first invoked listener returns
    /// `Some(event)` again, the next listener is invoked with `event`, and so on. If at some point a listener returns
    /// `None`, the chain ends and subsequent listeners are not invoked anymore.
    pub fn bootstrap<T>(&self, event: T, callback: fn(T) -> Option<T>) -> Result<(), T>
    where
        T: 'static,
    {
        // Register the listener
        if self.listen(callback).is_err() {
            return Err(event);
        };

        // Send the seed event
        self.send(event)
    }
    /// Sends an event to the event loop, returns `Err(event)` if the backlog is reached
    pub fn send<T>(&self, event: T) -> Result<(), T>
    where
        T: 'static,
    {
        // Insert the event
        let event_box = Box::new(event)?;
        if let Err(event_box) = self.events.scope(|events| events.push(event_box)) {
            return Err(event_box.into_inner().expect("failed to unwrap event"));
        };

        // Trigger a hardware event
        unsafe { runtime::_runtime_sendevent_ZMWrWpGO() };
        Ok(())
    }

    /// Enters the event loop
    pub fn enter(&self) -> ! {
        'event_loop: loop {
            // Wait for event
            let mut maybe_event_box = self.events.scope(|events| events.pop());
            if maybe_event_box.is_none() {
                // Wait for a hardware event and continue
                unsafe { runtime::_runtime_waitforevent_r3iRR3iR() };
                continue 'event_loop;
            }

            // Invoke matching event listeners
            let mut listeners = self.listeners.scope(|listeners| listeners.into_iter());
            // Iterate as long as we have a) an event to process and b) an event listener to test against
            while let (Some(event_box), Some(listener)) = (maybe_event_box.take(), listeners.next()) {
                // Check if the event type matches the callback's type
                let EventListener { type_id, callback_box, caller } = listener;
                if type_id == event_box.inner_type_id() {
                    // Call the callback and store the returned event box
                    maybe_event_box = caller(event_box, callback_box);
                }
            }
        }
    }

    /// Calls a callback with an event
    fn caller<T>(boxed_event: Box<STACKBOX_SIZE>, callback: CopyBox<FPTR_SIZE>) -> Option<Box<STACKBOX_SIZE>>
    where
        T: 'static,
    {
        // Recover the original types
        let event: T = boxed_event.into_inner().expect("failed to unwrap event");
        let callback: fn(T) -> Option<T> = callback.inner().expect("failed to unwrap callback");

        // Call the callback and box the result
        let event = callback(event)?;
        let boxed_event = Box::new(event).unwrap_or_else(|_| unreachable!("failed to re-box event"));
        Some(boxed_event)
    }
}
