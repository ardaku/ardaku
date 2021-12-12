struct System;

impl ardaku::System for System {
    fn sleep(&self) -> (ardaku::Event, u32) {
        todo!()
    }

    fn write(&self, byte: u32) {
        print!("{}", byte);
    }

    fn version(&self) -> u32 {
        0xDEADBEEF
    }

    fn reboot(&self) {
        std::process::exit(0);
    }
}

fn main() -> ardaku::Result<()> {
    let app_path = std::env::args().skip(1).next().expect("Provide wasm file!");
    let exe = std::fs::read(app_path).expect("Couldn't find file!");

    ardaku::start(System, &exe)
}
