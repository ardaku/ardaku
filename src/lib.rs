//! Dive-Kernel is a unikernel written in rust.

// TODO: enable for actually building a unikernel
// #![no_std]

/*type Void = u8;

mod data; // Data on the heap
mod future; // A state machine future
mod slice; // Variable number of a data on the heap

pub use future::Future;
pub use slice::Slice;

// Drivers
pub mod usb;*/

mod sr;
mod dive;

pub use sr::*;
pub use dive::*;

/// Entry point for the kernel.
pub extern "C" fn dive_main() {
    
}
