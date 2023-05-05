//! Defines required runtime-specific function stubs

extern "Rust" {
    /// Blocks until an event occurs (may wake spuriously); should be similar to ARM's `wfe` instruction
    pub(crate) fn _runtime_waitforevent_r3iRR3iR();
    /// Raises an event; should be similar to ARM's `sev` instruction
    pub(crate) fn _runtime_sendevent_ZMWrWpGO();
    /// Ensures that `code` is run exclusively, without being subject to race conditions or interrupts
    pub(crate) fn _runtime_threadsafe_NfpNM21J(code: &mut dyn FnMut());
}
