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
    parse::{Reader, Writer},
};

/// The system should implement these syscalls
pub trait System {
    /// Sleep until some event(s) happen.
    ///
    /// # Parameters
    ///  - `bytes`: Slice of bytes of WebAssembly module memory
    ///  - `size`: Capacity for number of `u32`s in ready list
    ///  - `data`: Pointer in bytes to ready list
    ///
    /// # Returns
    ///  - Length of overwritten ready list
    fn sleep(&self, bytes: &mut [u8], size: usize, data: usize) -> usize;

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
    ///  - `data`: Pointer to the UTF-8 buffer (`size: u32`, `reference: u32`)
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
    size: u32,
    data: u32,
    channel: u32,
    ready: u32,
}

/// Connect
#[derive(Debug)]
struct Connect {
    portals_size: u32,
    portals_data: u32,
    ready_capacity: u32,
    ready_data: u32,
}

/// Portal IDs
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
enum Portal {
    /// Logging API (stdout/printf)
    Log = 0,
    /// Developer command API (stdin/scanf)
    Prompt = 1,
    /// Set user information API (username, display name)
    Account,
    /// Get user information API (username, display name)
    User,
    /// Set system information API (system nickname, hostname)
    System,
    /// Get system information API (system nickname, hostname)
    Host,
    /// Set hardware features API (overclock, hardware time)
    Hardware,
    /// Get hardware features API (cpu / gpu specs)
    Platform,
    /// Task spawning API
    Spawn,
    /// Blocking task spawning API
    SpawnBlocking,
    /// MPMC Channel API
    Channel,
    /// Account API (create / delete users)
    Admin,
    ///
    Max,
}

struct ConnectedChannel<S: System> {
    portal: Portal,
    callback: fn(&mut S, u32, &mut [u8], u32, u32) -> bool,
}

fn fixme<S: System>(_: &mut S, _: u32, _: &mut [u8], _: u32, _: u32) -> bool {
    log::error!(target: "ardaku", "FIXME");

    true
}

fn prompt<S: System>(
    system: &mut S,
    ready: u32,
    bytes: &mut [u8],
    size: u32,
    data: u32,
) -> bool {
    log::trace!(target: "ardaku", "prompt");

    let size: usize = size.try_into().unwrap();
    let data: usize = data.try_into().unwrap();

    log::trace!(target: "ardaku", "prompt size: {size}, data: {data}");

    if size != 8 {
        todo!("Host trap: command size {size}");
    }

    let mut prompt_cmd = Reader::new(&bytes[data..][..size]);
    let capacity_ref: usize = prompt_cmd.u32().try_into().unwrap();
    let text_ref: usize = prompt_cmd.u32().try_into().unwrap();

    log::trace!(target: "ardaku", "prompt readline");

    system.read_line(ready, text_ref, capacity_ref);

    false
}

fn log<S: System>(
    system: &mut S,
    _ready: u32,
    bytes: &mut [u8],
    size: u32,
    data: u32,
) -> bool {
    let size: usize = size.try_into().unwrap();
    let data: usize = data.try_into().unwrap();

    if size != 16 {
        todo!("Host trap: command size {size}");
    }

    let mut log_cmd = Reader::new(&bytes[data..][..size]);
    let target_size: u8 = log_cmd.u16().try_into().unwrap_or(u8::MAX);
    let log_level: u16 = log_cmd.u16();
    let target_data: usize = log_cmd.u32().try_into().unwrap();
    let message_size: usize = log_cmd.u32().try_into().unwrap();
    let message_data: usize = log_cmd.u32().try_into().unwrap();

    log::trace!(target: "ardaku", "Message (data, size) = ({target_data}, {target_size})");
    let target = &bytes[target_data..][..usize::from(target_size)];
    let target = if let Ok(target) = core::str::from_utf8(target) {
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
        0 => {
            log::info!(target: "ardaku", "Panic triggered");
            system.log(message, Level::Error, target);
            todo!("Host trap: custom fatal");
        }
        1 => Level::Error,
        2 => Level::Warn,
        3 => Level::Info,
        4 => Level::Debug,
        5 => Level::Trace,
        _ => todo!("Host trap: invalid log level"),
    };

    system.log(message, level, target);

    true
}

impl<S: System> State<S> {
    fn bytes_and_state<'a>(
        caller: &'a mut Caller<'_, Self>,
    ) -> (&'a mut [u8], &'a mut State<S>) {
        let state = caller.data_mut();
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
        type Callback<S> = fn(&mut S, u32, &mut [u8], u32, u32) -> bool;

        let cap = connect.ready_capacity;
        let ptr = connect.ready_data;
        log::trace!(target: "ardaku", "Connect: cap {cap}, ptr {ptr:x}");

        self.ready_list = (connect.ready_capacity, connect.ready_data);

        let mut offset: usize = connect.portals_data.try_into().unwrap();
        for _ in 0..connect.portals_size {
            let mut reader = Reader::new(&bytes[offset..]);
            let portal = reader.u32();
            let (portal, callback): (_, Callback<S>) = match portal {
                0 => (Portal::Log, log::<S>),
                1 => (Portal::Prompt, prompt::<S>),
                2 => (Portal::Account, fixme::<S>),
                3 => (Portal::User, fixme::<S>),
                4 => (Portal::System, fixme::<S>),
                5 => (Portal::Host, fixme::<S>),
                6 => (Portal::Hardware, fixme::<S>),
                7 => (Portal::Platform, fixme::<S>),
                8 => (Portal::Spawn, fixme::<S>),
                9 => (Portal::SpawnBlocking, fixme::<S>),
                10 => (Portal::Channel, fixme::<S>),
                11 => (Portal::Admin, fixme::<S>),
                12.. => todo!("Host trap: invalid portal"),
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
                portals_size: reader.u32(),
                portals_data: reader.u32(),
                ready_capacity: reader.u32(),
                ready_data: reader.u32(),
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

            callback(&mut self.system, ready, bytes, size, data)
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
    let mut none_waiting = true;
    for _ in 0..size {
        let mut reader = Reader::new(&bytes[offset..]);
        let command = Command {
            size: reader.u32(),
            data: reader.u32(),
            channel: reader.u32(),
            ready: reader.u32(),
        };

        log::trace!(target: "ardaku", "DBG {command:?}");

        none_waiting &= state.execute(bytes, command);
        offset += 4 * core::mem::size_of::<u32>();
    }

    let ready_size = state.ready_list.0.try_into().unwrap();
    let ready_data = state.ready_list.1.try_into().unwrap();

    log::trace!(target: "ardaku", "Ready ({none_waiting})");

    if !none_waiting {
        state
            .system
            .sleep(bytes, ready_size, ready_data)
            .try_into()
            .unwrap()
    } else {
        let ready_list = &mut bytes[ready_data..][..ready_size * 4];
        let mut writer = Writer::new(ready_list);
        for _ in 0..ready_size {
            writer.u32(u32::MAX);
        }

        0
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
    let mut linker = <Linker<State<S>>>::new(&engine);
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
    store.data_mut().memory = MaybeUninit::new(memory);

    let run = instance
        .get_export(&store, "run")
        .and_then(Extern::into_func)
        .ok_or(Error::MissingRun)?
        .typed::<(), ()>(&mut store)
        .map_err(|_| Error::MissingRun)?;

    // And finally we can call the wasm!
    run.call(&mut store, ()).map_err(Error::Crash)?;

    //

    let current_pages = unsafe {
        store
            .data_mut()
            .memory
            .assume_init()
            .current_pages(&mut store)
            .to_bytes()
            .unwrap()
            / 65_536
    };

    log::info!(target: "ardaku", "Pages allocated at exit: {current_pages}");
    log::info!(target: "ardaku", " - As kB: {}", current_pages * 64);

    Ok(())
}
