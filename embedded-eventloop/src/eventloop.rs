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

/// An event listener with the associated type and caller
type EventListener<const SIZE: usize> = (TypeId, CopyBox<FPTR_SIZE>, fn(Box<SIZE>, CopyBox<FPTR_SIZE>));

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
    /// # Warning
    /// While it is possible to add multiple listeners for the same type `T`, only the first added listener will be called
    pub fn listen<T>(&self, callback: fn(T)) -> Result<(), fn(T)>
    where
        T: 'static,
    {
        // Create the caller
        let callback_box = CopyBox::new(callback).expect("cannot box function pointer");
        let caller: fn(Box<STACKBOX_SIZE>, CopyBox<FPTR_SIZE>) = Self::caller::<T>;
        let listener = (TypeId::of::<T>(), callback_box, caller);

        // Insert the listener
        if self.listeners.scope(|listeners| listeners.push(listener)).is_err() {
            return Err(callback);
        }
        Ok(())
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
            unsafe { runtime::_runtime_waitforevent_r3iRR3iR() };

            // Grab the next event
            let Some(event_box) = self.events.scope(|events| events.pop()) else {
                continue 'event_loop;
            };

            // Find a matching event handler
            let maybe_listener_caller = self.listeners.scope(|listeners| {
                // Find the first listener which type ID matches the event's type ID
                listeners.iter().find(|(type_id, _, _)| *type_id == event_box.inner_type_id()).copied()
            });

            // Call the listener
            if let Some((_, callback_box, caller)) = maybe_listener_caller {
                caller(event_box, callback_box);
            }
        }
    }

    /// Calls a callback with an event
    fn caller<T>(event: Box<STACKBOX_SIZE>, callback: CopyBox<FPTR_SIZE>)
    where
        T: 'static,
    {
        // Recover the original types and call the callback
        let event: T = event.into_inner().expect("failed to unwrap event");
        let callback: fn(T) = callback.inner().expect("failed to unwrap callback");
        callback(event);
    }
}
