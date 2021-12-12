#![no_std]

use wasmi::{
    FuncInstance, FuncRef, ImportsBuilder, Module, ModuleImportResolver,
    ModuleInstance, Signature, Trap, ValueType, Externals, RuntimeArgs, RuntimeValue
};
use core::num::NonZeroU32;

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
    service_log: Option<NonZeroU32>,
}

impl<S: System> State<S> {
    // Internal event implementation (exported to wasm module).
    unsafe fn event(&self, size: u32, data_constp: u32, done_mutp: u32) -> u32 {
        todo!()
        /*for i in 0..size {

        }*/
    }
}

extern "C" {
    fn puts(s: *const u8) -> i32;
}

impl<S: System> Externals for State<S> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> core::result::Result<Option<RuntimeValue>, Trap> {
        unsafe {
            puts(b"TEST\0".as_ptr());
        }

        match index {
            0 => {
                let size: i32 = args.nth(0);
                let data: i32 = args.nth(1);
                let done: i32 = args.nth(2);

                // Currently there are no async events.
                let ready: i32 = 0;
                Ok(Some(ready.into()))
            }
            _ => unreachable!(),
        }
    }
}

struct ArdakuResolver;

impl ModuleImportResolver for ArdakuResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &Signature,
    ) -> core::result::Result<FuncRef, wasmi::Error> {
        let func_ref = match field_name {
            "event" => FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::I32, ValueType::I32][..],
                    Some(ValueType::I32),
                ),
                0, // index 0
            ),
            _ => {
                return Err(wasmi::Error::Trap(wasmi::Trap::new(
                    wasmi::TrapKind::UnexpectedSignature,
                )));
            }
        };
        Ok(func_ref)
    }
}

/// Start an Ardaku application.  `exe` must be a .wasm file.
pub fn start<S: System>(system: S, exe: &[u8]) -> Result<()> {
    let mut state = State { system, service_log: None };
    let resolver = ArdakuResolver;
    let module = Module::from_buffer(&exe).map_err(|_| Error::InvalidWasm)?;
    let imports = ImportsBuilder::default().with_resolver("ardaku", &resolver);

    ModuleInstance::new(&module, &imports)
        .map_err(|_| Error::LinkerFailed)?
        .run_start(&mut state)
        .map_err(|t| Error::Crash(t))?;

    Ok(())
}
