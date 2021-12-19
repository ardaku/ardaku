use std::io::{self, Write};
use ardaku::{Device, DeviceKind};

struct System;

impl ardaku::System for System {
    fn sleep(&self) -> (ardaku::Event, u32) {
        todo!()
    }

    fn log(&self, line: &[u8]) {
        let mut stdout = io::stdout();
        let _ = stdout.write_all(line);
        let _ = stdout.write_all(b"\n");
        let _ = stdout.flush();
    }

    fn version(&self) -> u32 {
        0xDEADBEEF
    }

    fn reboot(&self) {
        std::process::exit(0);
    }

    fn connect(&self, _: DeviceKind) { todo!() }
    fn disconnect(&self, _: DeviceKind) { todo!() }
    fn register(&self, _: Device) { todo!() }
    fn deregister(&self, _: Device) { todo!() }
}

fn main() -> ardaku::Result<()> {
    let app_path = std::env::args().skip(1).next().expect("Provide wasm file!");
    let exe = std::fs::read(app_path).expect("Couldn't find file!");

    ardaku::start(System, &exe)
}
