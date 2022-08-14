#![no_std]

use core::mem::MaybeUninit;
use wasmi::core::Trap;
use wasmi::{Caller, Extern, Func, Linker, Memory, Module, Store};

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

    /// Write a line of text to stdout
    fn write(&self, line: &[u8]);

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
    /// Application does not export "ardaku" memory.
    MissingMemory,
    /// "run" function not exported
    MissingRun,
}

struct State<S: System> {
    memory: MaybeUninit<Memory>,
    system: S,
}

/// Asynchronous Request
fn ar<S>(mut caller: Caller<'_, State<S>>, size: u32, data: u32) -> u32
where
    S: System + 'static,
{
    let state = caller.host_data_mut();
    let memory = unsafe { state.memory.assume_init_mut() };

    todo!("{:?}", (size, data));
}

/// Run an Ardaku application.  `exe` must be a .wasm file.
pub fn run<S>(system: S, exe: &[u8]) -> Result<()>
where
    S: System + 'static,
{
    let engine = wasmi::Engine::default();
    let module = Module::new(&engine, exe).map_err(|_| Error::InvalidWasm)?;
    let mut store = Store::new(
        &engine,
        State {
            system,
            memory: MaybeUninit::uninit(),
        },
    );
    let async_request = Func::wrap(&mut store, ar);
    let mut linker = <Linker<State<S>>>::new();
    linker
        .define("daku", "ar", async_request)
        .map_err(|_| Error::LinkerFailed)?;
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|_| Error::InvalidWasm)?
        .ensure_no_start(&mut store)
        .map_err(|_| Error::InvalidWasm)?;
    let memory = instance
        .get_export(&mut store, "memory")
        .ok_or(Error::MissingMemory)?
        .into_memory()
        .ok_or(Error::MissingMemory)?;
    store.state_mut().memory = MaybeUninit::new(memory);

    let run = instance
        .get_export(&store, "run")
        .and_then(Extern::into_func)
        .ok_or(Error::MissingRun)?
        .typed::<(), (), _>(&mut store)
        .map_err(|_| Error::MissingRun)?;

    // And finally we can call the wasm!
    run.call(&mut store, ()).map_err(Error::Crash)?;

    Ok(())
}
