use std::{
    io::{self, BufRead, Write},
    sync::Mutex,
    task::Poll,
};

use log::Level;

struct System {
    read_line: Mutex<Option<(u32, usize, usize)>>,
}

// FIXME: Use smelling_salts with whisk channel

impl ardaku::System for System {
    fn sleep(
        &self,
        bytes: &mut [u8],
        ready_data: usize,
        ready_size: usize,
    ) -> usize {
        log::debug!(target: "demo", "sleep");

        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        let mut buffer = String::with_capacity(1024);
        loop {
            handle.read_line(&mut buffer).unwrap();
            let (ready, text, capacity) = self.read_line.lock().unwrap().unwrap();
            let mut reader = ardaku::parse::Reader::new(&bytes[text..]);
            let size = usize::try_from(reader.u32()).unwrap();
            let addr = usize::try_from(reader.u32()).unwrap();
           
            // Write size to memory
            let size = {
                let mut writer = ardaku::parse::Writer::new(&mut bytes[size..]);
                let size = buffer.len().min(capacity).try_into().unwrap();
                writer.u32(size);
                usize::try_from(size).unwrap()
            };

            // Write read line to memory
            {
                let buf = &mut bytes[addr..][..size];
                buf.copy_from_slice(buffer.as_bytes());
            }

            // Add to ready list
            {
                let ready_list = &mut bytes[ready_data..][..ready_size];
                let mut writer = ardaku::parse::Writer::new(ready_list);
                writer.u32(ready);
            }
        }
        0
    }

    fn log(&self, text: &str, level: Level, target: &str) {
        log::log!(target: target, level, "{text}")
    }

    fn read_line(
        &self,
        ready: u32,
        index: usize,
        length: usize,
    ) {
        let mut read_line = self.read_line.lock().unwrap();
        *read_line = Some((ready, index, length));
    }
}

fn main() -> ardaku::engine::Result {
    // Setup
    env_logger::init();

    let app_path = std::env::args().skip(1).next().expect("Provide wasm file!");
    let exe = std::fs::read(app_path).expect("Couldn't find file!");

    // Run app
    let system = System {
        read_line: Mutex::new(None),
    };

    ardaku::run(system, &exe)
}
