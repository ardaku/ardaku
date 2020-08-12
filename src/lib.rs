//! Dive-Kernel is a micro-unikernel written in rust.
//!
//! # Components
//! - Emu(lator) ("Drivers" - emulate subsystems from devices on the target)
//! - Sys(calls) (Some syscalls are implemented ontop of the emulator)
//! - Syn(chronicity) (The concurrency model is implemented with sycalls)

// Components
mod emu;
mod sys;
mod syn;


/*type Void = u8;

mod data; // Data on the heap
mod future; // A state machine future
mod slice; // Variable number of a data on the heap

pub use future::Future;
pub use slice::Slice;

// Drivers
pub mod usb;*/
