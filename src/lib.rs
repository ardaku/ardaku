#![no_std]

extern crate alloc;

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

/// Command
#[derive(Debug)]
struct Command {
    ready: u32,
    channel: u32,
    size: u32,
    data: u32,
}

/// Connect
#[derive(Debug)]
struct Connect {
    ready_capacity: u32,
    ready_data: u32,
    portals_size: u32,
    portals_data: u32,
}

fn le_u32(slice: &[u8]) -> u32 {
    u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]])
}

impl<S: System> State<S> {
    /// Connect channels
    fn connect(caller: &mut Caller<'_, Self>, mem: Memory, connect: Connect) {
        let _bytes = mem.data_mut(caller);
        todo!("{connect:?}");
    }

    /// Execute a command from an asynchronous request
    fn execute(caller: &mut Caller<'_, Self>, mem: Memory, command: Command) {
        if command.channel == 0 {
            let bytes = mem.data_mut(&mut *caller);
            // FIXME: Trigger a trap if doesn't match
            assert_eq!(command.size, 16);
            let offset: usize = command.data.try_into().unwrap();
            let connect = Connect {
                ready_capacity: le_u32(&bytes[offset..]),
                ready_data: le_u32(&bytes[offset+4..]),
                portals_size: le_u32(&bytes[offset+8..]),
                portals_data: le_u32(&bytes[offset+12..]),
            };

            Self::connect(caller, mem, connect);
        } else {
            todo!();
        }
    }
}

/// Asynchronous Request
fn ar<S>(mut caller: Caller<'_, State<S>>, size: u32, data: u32) -> u32
where
    S: System + 'static,
{
    let state = caller.host_data_mut();
    let memory = unsafe { state.memory.assume_init_mut() }.clone();
    let data: usize = data.try_into().unwrap();

    let mut offset = data;
    for _ in 0..size {
        let bytes = memory.data_mut(&mut caller);
        let command = Command {
            ready: le_u32(&bytes[offset..]),
            channel: le_u32(&bytes[offset+4..]),
            size: le_u32(&bytes[offset+8..]),
            data: le_u32(&bytes[offset+12..]),
        };
        offset += 16;

        State::<S>::execute(&mut caller, memory, command);
    }

    todo!("Wait for a command to complete")
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
