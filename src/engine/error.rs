//! Errors

use wasmi::core::Trap;

/// WebAssembly Engine Result
pub type Result<T = (), E = Error> = core::result::Result<T, E>;

/// WebAssembly Engine Error
#[derive(Debug)]
pub enum Error {
    /// The WASM file is invalid
    InvalidWasm,
    /// Memory / function linking failed
    LinkerFailed,
    /// Application has crashed from one of the various traps
    Crash(Trap),
    /// Application does not export "ardaku" memory.
    MissingMemory,
    /// "run" function not exported
    MissingRun,
}
