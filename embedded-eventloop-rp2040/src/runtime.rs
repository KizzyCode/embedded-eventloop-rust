//! Provides the runtime specific functions for an rp2040 platform

use cortex_m::asm;

/// Blocks until an event occurs (may wake spuriously); should be similar to ARM's `wfe` instruction
#[no_mangle]
#[doc(hidden)]
#[allow(non_snake_case)]
pub fn _runtime_waitforevent_r3iRR3iR() {
    asm::wfe();
}

/// Raises an event; should be similar to ARM's `sev` instruction
#[no_mangle]
#[doc(hidden)]
#[allow(non_snake_case)]
pub fn _runtime_sendevent_ZMWrWpGO() {
    asm::sev()
}

/// Ensures that `code` is run exclusively, without being subject to race conditions or interrupts
#[no_mangle]
#[doc(hidden)]
#[allow(non_snake_case)]
pub fn _runtime_threadsafe_NfpNM21J(code: &mut dyn FnMut()) {
    critical_section::with(|_| code())
}
