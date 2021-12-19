//! Low-level interface for the Ardaku system.  This is similar to how the libc
//! and windows/winapi crates are used for unix-derivatives and Windows (or
//! web-sys for targetting the web).
//!
//! Examples will use the pasts async runtime, because the system is
//! asynchronous in many of it's APIs.  Other runtimes may or may not work.

use std::mem::transmute;
use std::task::Waker;

pub use rdaku::{Command, Device, Service};

// None: Device is ready (output written)
// Some: Device is not ready yet.
#[derive(Clone)]
struct DeviceWakeStatus(Option<Waker>); // should be 64 bits.

/// Thread-local data (can't use std lib thread local because non-functional on
/// web assembly).
#[derive(Clone)]
struct ThreadState {
    event_queue: [u32; 250],
    command_queue: Option<Vec<Command>>,
    /// First 32 indices are reserved for use by this library, all after are
    /// dynmically allocated.
    devices: Option<Vec<DeviceWakeStatus>>,
}

/// Max out at 64 threads (this reserves one wasm page for thread-local data).
static mut THREADS: [ThreadState; 64] = unsafe { transmute([[0u8; 1024]; 64]) };

/// Device ID of the current running thread.  This is a magic variable because
/// depending on which WASM thread, it has a different value.
static mut THREAD_ID: usize = 0;

/// Process commands in command queue and return number of events in event queue
fn event(cmds: &[Command]) -> u8 {
    #[link(wasm_import_module = "ardaku")]
    extern "C" {
        fn event(size: u32, data: u32, done: u32) -> u32;
    }

    unsafe {
        let evq: *mut ThreadState = &mut THREADS[THREAD_ID];
        let done = u32::from_ne_bytes((evq as usize).to_ne_bytes());
        event(
            u32::from_ne_bytes(cmds.len().to_ne_bytes()),
            u32::from_ne_bytes((cmds.as_ptr() as usize).to_ne_bytes()),
            done,
        )
        .try_into()
        .unwrap_or(u8::MAX)
    }
}

/// Process command queue and go to sleep until ready.
pub fn sleep() {
    let num_events = unsafe { THREADS[THREAD_ID].command_queue.as_mut() }
        .map(|x| {
            let n = event(x.as_slice());
            x.clear();
            n
        })
        .unwrap_or(0);

    // Process asynchronous events.
    for i in 0..num_events.into() {
        unsafe {
            let device = THREADS[THREAD_ID].event_queue[i];
            if let Some(ref mut devices) = THREADS[THREAD_ID].devices {
                // Set to waker to `None` (meaning it's ready) and wake.
                if let Some(waker) = devices[device as usize].0.take() {
                    waker.wake();
                }
            }
        }
    }
}

///
async fn connect(service: Service) {
    sleep(/*&[Command {
        channel_id: 0, // Connector
        channel_id_new: 1 + (service as u32),
        message_size: 0,
        message_data: service as u32, // SERVICE ID
    }]*/);
}

/*
/// Save a message to the log.
pub async fn log(message: &str) {
    if uninit(Service::Logging) {
        connect(Service::Logging).await;
    }
}*/

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
