#![no_std]
#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub mod runtime;

// Re-export everything
extern crate embedded_eventloop;
pub use embedded_eventloop::*;
