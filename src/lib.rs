#![no_std]

use core::num::NonZeroU32;
use wasmi::{
    ExternVal, Externals, FuncInstance, FuncRef, ImportsBuilder, MemoryRef,
    Module, ModuleImportResolver, ModuleInstance, RuntimeArgs, RuntimeValue,
    Signature, Trap, ValueType,
};

pub use rdaku::Device;

/// An Ardaku event.
#[derive(Debug)]
pub enum Event {
    /// Device has been connected, returns device and hardware identifier and
    /// name.
    Connect(DeviceKind, Device, u64),

    /// Timer activated
    Timer(Device),

    /// USB message FIXME: TODO
    Usb(Device, *const ()),

    /// Keyboard message; 8-bit key ID and whether or not key is pressed.
    Key(Device, u8, bool),

    /// Pointer event; 32-bit eventID, 32-bit value (xy 16:16)
    /// FIXME: First u32 param → enum
    Pointer(Device, u32, u32),

    /// Controller message; 32-bit eventID, 32-bit value
    Controller(Device, u32, u32),

    /// Get microphones input (up to 8 channels of 32-sample chunks).
    ///
    /// Should follow FLAC/SMPTE/ITU-R recommendations for 7.1
    ///  - FrontL, FrontR, Front, Lfe, BackL, BackR, Left, Right
    Microphone(Device, *const [[f32; 32]; 8]),

    /// Request speaker output (up to 8 channels of 32-sample chunks).
    ///
    /// Should follow FLAC/SMPTE/ITU-R recommendations for 7.1
    ///  - FrontL, FrontR, Front, Lfe, BackL, BackR, Left, Right
    Speakers(Device, *mut [[f32; 32]; 8]),

    /// Camera capture FIXME: TODO
    Camera(Device, *const ()),

    /// Refresh monitor, phone screen, other display etc. FIXME: TODO
    Screen(Device, *const ()),

    /// MIDI device event. FIXME: TODO
    Midi(Device, *const ()),

    /// Serial device command.  Size and buffer.
    Serial(Device, usize, *const u8),

    /// GPIO device input interrupt. FIXME: TODO
    Gpio(Device, *const ()),

    /// File write complete. FIXME: TODO
    Storage(Device, *const ()),
}

/// Types of devices that can be hooked up to a computer and send async events.
#[derive(Debug)]
pub enum DeviceKind {
    /// Find timer device
    Timer,
    /// Find devices with generic USB driver
    Usb,
    /// Find keyboards
    Keyboard,
    /// Find pointer devices (touchscreen, trackpad, mouse, etc.)
    Pointer,
    /// Find W3C Standard Gamepad compliant controllers and other joysticks.
    Controller,
    /// Find microphones, or other audio input devices.
    Microphone,
    /// Find speakers, or other audio output devices
    Speakers,
    /// Find webcams, phone cameras, etc.
    Camera,
    /// Find monitor, phone screen, other display etc.
    Screen,
    /// Find MIDI device.
    Midi,
    /// Find serial device.
    Serial,
    /// Find GPIO device.
    Gpio,
    /// Find file storage device.
    Storage,
}

/// The system should implement these syscalls
pub trait System {
    /// Register a connector, this affects what can be returned from `sleep()`.
    ///
    /// This may involve enabling specific interrupts, or enabling a response.
    fn connect(&self, kind: DeviceKind);

    /// Stop checking for new devices to connect to of specific kind.
    fn disconnect(&self, kind: DeviceKind);

    /// Register events for device
    fn register(&self, device: Device);

    /// Deregister events for device
    fn deregister(&self, device: Device);

    /// Sleep until a registered event happens.
    fn sleep(&self) -> (Event, u32);

    /// Log a line of text (stdout)
    fn log(&self, line: &[u8]);

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
                let channel_id =
                    u32::from_le_bytes(memory[ptr..][..4].try_into().unwrap());
                let channel_id_new = u32::from_le_bytes(
                    memory[ptr + 4..][..4].try_into().unwrap(),
                );
                let message_size: usize = u32::from_le_bytes(
                    memory[ptr + 8..][..4].try_into().unwrap(),
                )
                .try_into()
                .unwrap();
                let message_data_ptr: usize = u32::from_le_bytes(
                    memory[ptr + 12..][..4].try_into().unwrap(),
                )
                .try_into()
                .unwrap();

                let message = &memory[message_data_ptr..][..message_size];

                // Check for special case; channel_id == 0
                if channel_id == 0 {
                    #[allow(clippy::single_match)]
                    match message {
                        b"log" => {
                            self.service_log = NonZeroU32::new(channel_id_new)
                        }
                        _ => { /* ignore unknown services */ }
                    }
                } else if let Some(log) = self.service_log {
                    if log.get() == channel_id {
                        self.system.log(message);
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

    let mut state = State {
        system,
        service_log: None,
        memory,
    };

    instance.run_start(&mut state).map_err(Error::Crash)?;

    Ok(())
}
