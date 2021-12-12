// #![no_std]

use wasmi::{
    FuncInstance, FuncRef, ImportsBuilder, Module, ModuleImportResolver,
    ModuleInstance, Signature, Trap, ValueType, Externals, RuntimeArgs, RuntimeValue, ExternVal, MemoryRef
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
}

struct State<S: System> {
    system: S,
    memory: MemoryRef,
    service_log: Option<NonZeroU32>,
}

impl<S: System> Externals for State<S> {
    fn invoke_index(
        &mut self,
        _index: usize, // Always 0
        args: RuntimeArgs,
    ) -> core::result::Result<Option<RuntimeValue>, Trap> {
        let size: u32 = args.nth(0);
        let data: u32 = args.nth(1);
        let _done: u32 = args.nth(2); // No async events yet

        // FIXME: Needs some refactoring
        self.memory.with_direct_access_mut(|memory: &mut [u8]| {
            for i in 0..size {
                let ptr: usize = (data + i * 16).try_into().unwrap();

                // WASM is little-endian
                let channel_id = u32::from_le_bytes(
                    memory[ptr..][..4].try_into().unwrap()
                );
                let channel_id_new = u32::from_le_bytes(
                    memory[ptr+4..][..4].try_into().unwrap()
                );
                let message_size: usize = u32::from_le_bytes(
                    memory[ptr+8..][..4].try_into().unwrap()
                ).try_into().unwrap();
                let message_data_ptr: usize = u32::from_le_bytes(
                    memory[ptr+12..][..4].try_into().unwrap()
                ).try_into().unwrap();

                let message = &memory[message_data_ptr..][..message_size];

                // Check for special case; channel_id == 0
                if channel_id == 0 {
                    match message {
                        b"log" => self.service_log = NonZeroU32::new(channel_id_new),
                        _ => { /* ignore unknown services */ },
                    }
                } else if let Some(log) = self.service_log {
                    if log.get() == channel_id {
                        self.system.write(message);
                        if channel_id_new != channel_id {
                            self.service_log = NonZeroU32::new(channel_id_new);
                        }
                    }
                }
            }
        });

        // Currently there are no async events to return.
        let ready: u32 = 0;
        Ok(Some(ready.into()))
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
    let resolver = ArdakuResolver;
    let module = Module::from_buffer(&exe).map_err(|_| Error::InvalidWasm)?;
    let imports = ImportsBuilder::default().with_resolver("ardaku", &resolver);

    let instance = ModuleInstance::new(&module, &imports)
        .map_err(|_| Error::LinkerFailed)?;

    let memory = match instance
        .not_started_instance()
        .export_by_name("ardaku")
        .ok_or(Error::MissingMemory)?
    {
        ExternVal::Memory(mem) => mem,
        _ => return Err(Error::MissingMemory),
    };

    let mut state = State { system, service_log: None, memory };

    instance.run_start(&mut state)
        .map_err(|t| Error::Crash(t))?;

    Ok(())
}
