//! Provides the runtime specific functions for an rp2040 platform

use cortex_m::asm;

/// Blocks until an event occurs (may wake spuriously); should be similar to ARM's `wfe` instruction
#[no_mangle]
#[doc(hidden)]
pub fn _eventloop_0_1_wait_for_event() {
    asm::wfe();
}

/// Raises an event; should be similar to ARM's `sev` instruction
#[no_mangle]
#[doc(hidden)]
pub fn _eventloop_0_1_send_event() {
    asm::sev()
}

/// Ensures that `code` is run exclusively, without being subject to race conditions or interrupts
#[no_mangle]
#[doc(hidden)]
pub fn _eventloop_0_1_threadsafe(code: &mut dyn FnMut()) {
    critical_section::with(|_| code())
}
