#![no_std]

extern crate alloc;

use core::mem::MaybeUninit;
use core::task::Poll;
use wasmi::core::Trap;
use wasmi::{Caller, Extern, Func, Linker, Memory, Module, Store};
use log::Level;

/// The system should implement these syscalls
pub trait System {
    /// Sleep until some event(s) happen.
    ///
    /// # Returns
    ///  - Length of ready list
    ///
    /// # Safety
    ///  - Undefined behavior if return value != written length of ready list
    unsafe fn sleep(&self, memory: &mut [u8], index: usize, length: usize) -> usize;

    /// Write a message to the logs.
    fn log(&self, text: &str, level: Level, target: &str);

    /// Read a line of valid UTF-8 to the buffer, not including the newline
    /// character.
    ///
    /// # Returns
    ///  - `Ok(num_of_bytes_read)`
    ///  - `Err(num_of_bytes_required)`
    ///
    /// # Safety
    ///  - Undefined behavior if implementation writes invalid UTF-8.
    ///  - Undefined behavior if bytes written != `Ok(num_of_bytes_read)`
    unsafe fn read_line(&self, ready: u32, index: usize, length: usize);
}

/// I/O Result
pub type IoResult = core::result::Result<usize, usize>;

/// Ardaku Result
pub type Result<T = ()> = core::result::Result<T, Error>;

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
    ready_list: (u32, u32),
    portals: [bool; Portal::Max as u32 as usize],
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

/// Portal IDs
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
enum Portal {
    /// Task spawning API
    Spawn = 0,
    /// Blocking task spawning API
    SpawnBlocking = 1,
    /// Logging API (stdout/printf)
    Log = 2,
    /// Developer command API (stdin/scanf)
    Prompt = 3,
    /// MPMC Channel API
    Channel = 4,
    /// 
    Max = 5,
}

fn le_u32(slice: &[u8]) -> u32 {
    u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]])
}

impl<S: System> State<S> {
    /// Connect channels
    fn connect(caller: &mut Caller<'_, Self>, mem: Memory, connect: Connect) {
        let state = caller.host_data_mut();
       
        state.ready_list = (connect.ready_capacity, connect.ready_data);
       
        let mut offset: usize = connect.portals_data.try_into().unwrap();
        for _ in 0..connect.portals_size {
            let bytes = mem.data_mut(&mut *caller);
            let portal = le_u32(&bytes[offset..]);
            let portal = match portal {
                0 => Portal::Spawn,
                1 => Portal::SpawnBlocking,
                2 => Portal::Log,
                3 => Portal::Prompt,
                4 => Portal::Channel,
                5..=u32::MAX => todo!("Host trap: invalid portal"),
            };
            offset += 4;
            let state = caller.host_data_mut();
            state.portals[portal as u32 as usize] = true;
            log::trace!(target: "ardaku", "Connect portal: {portal:?}");
        }
    }

    /// Execute a command from an asynchronous request
    fn execute(caller: &mut Caller<'_, Self>, mem: Memory, command: Command) -> bool {
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
            true
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
    log::trace!(target: "ardaku", "Syscall ({size} commands)");

    let state = caller.host_data_mut();
    let memory = unsafe { state.memory.assume_init_mut() }.clone();
    let data: usize = data.try_into().unwrap();

    let mut offset = data;
    let mut ready_immediately = 0;
    for _ in 0..size {
        let bytes = memory.data_mut(&mut caller);
        let command = Command {
            ready: le_u32(&bytes[offset..]),
            channel: le_u32(&bytes[offset+4..]),
            size: le_u32(&bytes[offset+8..]),
            data: le_u32(&bytes[offset+12..]),
        };
        offset += 16;

        ready_immediately += u32::from(u8::from(State::<S>::execute(&mut caller, memory, command)));
    }

    let state = caller.host_data_mut();
    // let bytes = memory.data_mut(&mut caller);

    if ready_immediately == 0 {
        unsafe {
            state.system.sleep(&mut [], state.ready_list.0.try_into().unwrap(), state.ready_list.1.try_into().unwrap()).try_into().unwrap()
        }
    } else {
        ready_immediately
    }
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
            ready_list: (0, 0),
            portals: [false; Portal::Max as u32 as usize],
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
