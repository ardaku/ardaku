#![no_std]

extern crate alloc;

pub mod engine;
pub mod parse;

use alloc::vec::Vec;
use core::mem::MaybeUninit;

use log::Level;
use wasmi::{Caller, Extern, Func, Linker, Memory, Module, Store};

use self::{
    engine::{Error, Result as EngineResult},
    parse::Reader,
};

/// The system should implement these syscalls
pub trait System {
    /// Sleep until some event(s) happen.
    ///
    /// # Parameters
    ///  - `bytes`: Slice of bytes of WebAssembly module memory
    ///  - `data`: Pointer in bytes to ready list
    ///  - `size`: Capacity for number of `u32`s in ready list
    ///
    /// # Returns
    ///  - Length of overwritten ready list
    fn sleep(&self, bytes: &mut [u8], data: usize, size: usize) -> usize;

    /// Write a message to the logs.
    ///
    /// # Parameters
    ///  - `text`: The text to print to the logs
    ///  - `level`: The log level
    ///  - `target`: The log target
    fn log(&self, text: &str, level: Level, target: &str);

    /// Spawn task of reading a line of valid UTF-8 to the buffer, not including
    /// the newline character.
    ///
    /// When task is ready, append `ready` to ready list, and if
    ///  - Capacity big enough: Overwrite buffer and new smaller size
    ///  - Capacity too small: Overwrite required size, buffer untouched
    ///
    /// # Parameters
    ///  - `ready`: Ready identifier to be written into the ready list when read
    ///  - `data`: Pointer to the bytes in the UTF-8 buffer
    ///  - `size`: Pointer to the capacity of the UTF-8 buffer (in bytes)
    fn read_line(&self, ready: u32, data: usize, size: usize);
}

struct State<S: System> {
    memory: MaybeUninit<Memory>,
    system: S,
    ready_list: (u32, u32),
    portals: [bool; Portal::Max as u32 as usize],
    // Channel IDs that can be reclaimed
    drop_channels: Vec<u32>,
    // Next channel ID
    next_channel: u32,
    // Connected channels
    conn_channels: Vec<Option<ConnectedChannel<S>>>,
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

struct ConnectedChannel<S: System> {
    portal: Portal,
    callback: fn(&mut S, &mut [u8], u32, u32),
}

fn fixme<S: System>(_: &mut S, _: &mut [u8], _: u32, _: u32) {
    log::error!(target: "ardaku", "FIXME");
}

fn log<S: System>(system: &mut S, bytes: &mut [u8], size: u32, data: u32) {
    let size: usize = size.try_into().unwrap();
    let data: usize = data.try_into().unwrap();

    if size < 9 {
        todo!("Host trap: command size");
    }

    let mut log_cmd = Reader::new(&bytes[data..]);
    let message_size: usize = log_cmd.u32().try_into().unwrap();
    let message_data: usize = log_cmd.u32().try_into().unwrap();
    let log_level = log_cmd.u8();
    let target = if let Ok(target) = log_cmd.str(size) {
        target
    } else {
        todo!("Host trap: invalid utf8 (target)");
    };

    log::trace!(target: "ardaku", "Log portal: target={target}");

    log::trace!(target: "ardaku", "Message (data, size) = ({message_data}, {message_size})");

    let message = &bytes[message_data..][..message_size];
    let message = if let Ok(message) = core::str::from_utf8(message) {
        message
    } else {
        todo!("Host trap: invalid utf8 (message)");
    };

    let level = match log_level {
        0 => Level::Trace,
        1 => Level::Debug,
        2 => Level::Info,
        3 => Level::Warn,
        4 => Level::Error,
        _ => todo!("Host trap: invalid log level"),
    };

    system.log(message, level, target);
}

impl<S: System> State<S> {
    fn bytes_and_state<'a>(
        caller: &'a mut Caller<'_, Self>,
    ) -> (&'a mut [u8], &'a mut State<S>) {
        let state = caller.host_data_mut();
        let memory = unsafe { state.memory.assume_init() };
        memory.data_and_store_mut(caller)
    }

    /// Allocate a channel
    fn channel(&mut self) -> u32 {
        if let Some(channel_id) = self.drop_channels.pop() {
            channel_id
        } else {
            let channel_id = self.next_channel;
            self.next_channel += 1;
            channel_id
        }
    }

    /// Connect channels
    fn connect(&mut self, bytes: &mut [u8], connect: Connect) {
        type Callback<S> = fn(&mut S, &mut [u8], u32, u32);

        self.ready_list = (connect.ready_capacity, connect.ready_data);

        let mut offset: usize = connect.portals_data.try_into().unwrap();
        for _ in 0..connect.portals_size {
            let mut reader = Reader::new(&bytes[offset..]);
            let portal = reader.u32();
            let (portal, callback): (_, Callback<S>) = match portal {
                0 => (Portal::Spawn, fixme::<S>),
                1 => (Portal::SpawnBlocking, fixme::<S>),
                2 => (Portal::Log, log::<S>),
                3 => (Portal::Prompt, fixme::<S>),
                4 => (Portal::Channel, fixme::<S>),
                5..=u32::MAX => todo!("Host trap: invalid portal"),
            };
            self.portals[portal as u32 as usize] = true;
            let channel_id = self.channel();
            self.conn_channels.resize_with(
                usize::try_from(self.next_channel).unwrap(),
                || None,
            );
            self.conn_channels[usize::try_from(channel_id).unwrap()] =
                Some(ConnectedChannel { portal, callback });

            for (src, dst) in channel_id
                .to_le_bytes()
                .into_iter()
                .zip(bytes[offset..].iter_mut())
            {
                *dst = src;
            }
            log::trace!(target: "ardaku", "Connect portal: {portal:?} (Ch{channel_id})");

            offset += core::mem::size_of::<u32>();
        }
    }

    /// Execute a command from an asynchronous request
    fn execute(&mut self, bytes: &mut [u8], command: Command) -> bool {
        if command.channel == 0 {
            // FIXME: Trigger a trap if doesn't match
            assert_eq!(command.size, 16);
            let offset: usize = command.data.try_into().unwrap();
            let mut reader = Reader::new(&bytes[offset..]);
            let connect = Connect {
                ready_capacity: reader.u32(),
                ready_data: reader.u32(),
                portals_size: reader.u32(),
                portals_data: reader.u32(),
            };

            self.connect(bytes, connect);
            true
        } else {
            let Command {
                channel,
                size,
                data,
                ready,
            } = command;
            let len = self.conn_channels.len();
            log::trace!(target: "ardaku", "Ch{channel}: {len:?}");
            let (portal, callback) = if let Some(ref cc) =
                self.conn_channels[usize::try_from(channel).unwrap()]
            {
                (cc.portal, cc.callback)
            } else {
                todo!("Host trap: invalid channel");
            };

            if !self.portals[portal as usize] {
                todo!("Host trap: unsupported portal");
            }

            log::trace!(target: "ardaku", "Ch{channel}: {portal:?}");

            callback(&mut self.system, bytes, size, data);

            true // Ready immediately

            /*let current_pages = unsafe {
                mem.current_pages(&mut *caller).0
            };
            todo!("pages: {current_pages}");*/
        }
    }
}

fn dbg<S>(mut caller: Caller<'_, State<S>>, size: u32, text: u32)
where
    S: System + 'static,
{
    let (bytes, _state) = State::bytes_and_state(&mut caller);
    let string = core::str::from_utf8(
        &bytes[usize::try_from(text).unwrap()..]
            [..usize::try_from(size).unwrap()],
    )
    .expect("daku debug failure");

    log::trace!(target: "daku-dbg", "{string}");
}

/// Asynchronous Request
fn ar<S>(mut caller: Caller<'_, State<S>>, size: u32, data: u32) -> u32
where
    S: System + 'static,
{
    let (bytes, state) = State::bytes_and_state(&mut caller);

    log::trace!(target: "ardaku", "Syscall ({size} commands)");

    let mut offset: usize = data.try_into().unwrap();
    let mut ready_immediately = 0;
    for _ in 0..size {
        let mut reader = Reader::new(&bytes[offset..]);
        let command = Command {
            ready: reader.u32(),
            channel: reader.u32(),
            size: reader.u32(),
            data: reader.u32(),
        };

        log::trace!(target: "ardaku", "DBG {command:?}");

        ready_immediately += u32::from(u8::from(state.execute(bytes, command)));
        offset += 4 * core::mem::size_of::<u32>();
    }

    if ready_immediately == 0 {
        state
            .system
            .sleep(
                &mut [],
                state.ready_list.0.try_into().unwrap(),
                state.ready_list.1.try_into().unwrap(),
            )
            .try_into()
            .unwrap()
    } else {
        ready_immediately
    }
}

/// Run an Ardaku application.  `exe` must be a .wasm file.
pub fn run<S>(system: S, exe: &[u8]) -> EngineResult
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
            drop_channels: Vec::new(),
            next_channel: 1,
            conn_channels: Vec::new(),
        },
    );
    let async_request = Func::wrap(&mut store, ar);
    let debug = Func::wrap(&mut store, dbg);
    let mut linker = <Linker<State<S>>>::new();
    linker
        .define("daku", "ar", async_request)
        .map_err(|_| Error::LinkerFailed)?
        .define("daku", "dbg", debug)
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
