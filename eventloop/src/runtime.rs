//! Defines requires runtime-specific function stubs

extern "Rust" {
    /// Blocks until an event occurs (may wake spuriously); should be similar to ARM's `wfe` instruction
    pub(crate) fn _eventloop_0_1_wait_for_event();
    /// Raises an event; should be similar to ARM's `sev` instruction
    pub(crate) fn _eventloop_0_1_send_event();
    /// Ensures that `code` is run exclusively, without being subject to race conditions or interrupts
    pub(crate) fn _eventloop_0_1_threadsafe(code: &mut dyn FnMut());
}
