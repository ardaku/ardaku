use std::{io::BufRead, sync::Mutex};

use log::Level;

struct System {
    read_line: Mutex<Option<(u32, usize, usize)>>,
    pre_queued: Mutex<Option<String>>,
}

// FIXME: Use smelling_salts with whisk channel

impl ardaku::System for System {
    fn sleep(
        &self,
        bytes: &mut [u8],
        ready_size: usize,
        ready_data: usize,
    ) -> usize {
        log::debug!(target: "demo", "READY DATA: {ready_data:x} ({ready_size})");

        log::debug!(target: "demo", "sleep");

        let stdin = std::io::stdin();
        let mut handle = stdin.lock();

        // Blocking (FIXME) line reading, returns 1 ready event
        {
            let buffer =
                if let Some(buf) = self.pre_queued.lock().unwrap().take() {
                    buf
                } else {
                    let mut buffer = String::with_capacity(1024);
                    handle.read_line(&mut buffer).unwrap();
                    if buffer.ends_with('\n') {
                        buffer.pop();
                    }
                    buffer
                };
            let (ready, text, capptr) = self.read_line.lock().unwrap().unwrap();
            let mut reader = ardaku::parse::Reader::new(&bytes[text..]);
            let _size = usize::try_from(reader.u32()).unwrap();
            let addr = usize::try_from(reader.u32()).unwrap();
            let mut reader = ardaku::parse::Reader::new(&bytes[capptr..]);
            let capacity = usize::try_from(reader.u32()).unwrap();

            if capacity < buffer.len() {
                // Write required capacity to memory
                let mut writer =
                    ardaku::parse::Writer::new(&mut bytes[capptr..]);
                let size = buffer.len().try_into().unwrap();
                writer.u32(size);

                // Store buffer for re-use since WASM doesn't own it yet
                *self.pre_queued.lock().unwrap() = Some(buffer);
            } else {
                // Write size to memory
                let size = {
                    let mut writer =
                        ardaku::parse::Writer::new(&mut bytes[text..]);
                    let size = buffer.len().min(capacity).try_into().unwrap();
                    writer.u32(size);
                    usize::try_from(size).unwrap()
                };

                log::debug!(target: "demo", "Copying {size} bytes...");

                // Write read line to memory
                {
                    let buf = &mut bytes[addr..][..size];
                    buf.copy_from_slice(buffer.as_bytes());
                }
            }

            log::debug!(target: "demo", "Add to ready list");

            // Add to ready list
            {
                let ready_list = &mut bytes[ready_data..][..ready_size * 4];
                let mut writer = ardaku::parse::Writer::new(ready_list);
                writer.u32(ready);
            }

            1
        }
    }

    fn log(&self, text: &str, level: Level, target: &str) {
        log::log!(target: target, level, "{text}")
    }

    fn read_line(&self, ready: u32, index: usize, length: usize) {
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
        pre_queued: Mutex::new(None),
    };

    ardaku::run(system, &exe)
}
