#![no_std]

use wasmi::{Module, NopExternals, Trap, ImportsBuilder, ModuleInstance};

/// An Ardaku event.
#[repr(u32)]
#[derive(Debug)]
pub enum Event {
    /// Read UTF-32 character from stdin
    Read = 0u32,
}

/// The system should implement these syscalls
pub trait System {
    /// Sleep until an event happens.
    fn sleep(&self) -> (Event, u32);

    /// Write UTF-32 character to stdout
    fn write(&self, byte: u32);

    /// Return kernel version
    fn version(&self) -> u32;

    /// Reboot the system
    fn reboot(&self);
}

/// Ardaku Result
pub type Result<T> = core::result::Result<T, Error>;

/// An error in Ardaku
#[derive(Debug)]
pub enum Error {
    /// The WASM file is invalid
    InvalidWasm,
    /// Memory / function linking failed
    LinkerFailed,
    /// Application has crashed from one of the various traps
    Crash(Trap),
}

/// Message from WASM.
#[repr(C, packed)]
struct Message {
    /// Channel to send a message on (0 is special "connector" channel)
    pub channel_id: i32,

    /// Channel ID is a user-chosen index into an array of function references.
    /// (set to 0 to disconnect `channel_id`)
    pub channel_id_new: u32,

    /// Size of message (in bytes)
    pub message_size: u32,

    /// Message data.
    pub message_data: *mut u8,
}

struct State<S: System> {
    system: S,
}

impl<S: System> State<S> {
    // Internal event implementation (exported to wasm module).
    unsafe fn event(&self, size: u32, data_constp: u32, done_mutp: u32) -> u32 {
        todo!()
        /*for i in 0..size {
            
        }*/
    }
}

/// Start an Ardaku application.  `exe` must be a .wasm file.
pub fn start<S>(system: S, exe: &[u8]) -> Result<()> {
    let module = Module::from_buffer(&exe).map_err(|_| Error::InvalidWasm)?;
    let imports = ImportsBuilder::default();

    ModuleInstance::new(&module, &imports)
        .map_err(|_| Error::LinkerFailed)?
        .run_start(&mut NopExternals)
        .map_err(|t| Error::Crash(t))?;
        
    Ok(())
}
