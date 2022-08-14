use std::io::{self, Write};
use std::sync::Mutex;
use std::task::Poll;
use std::io::BufRead;
use ardaku::IoResult;
use log::Level;

struct System {
    buffer: Mutex<String>,
    read_line: Mutex<Option<(u32, usize, usize)>>,
}

// FIXME: Use smelling_salts with whisk channel

impl ardaku::System for System {
    unsafe fn sleep(&self, memory: &mut [u8], index: usize, length: usize) -> usize {
        let ready_index = index;
        let ready_length = length;

        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        let mut buffer = self.buffer.lock().unwrap();
        handle.read_line(&mut buffer).unwrap();
        let mut read_line = self.read_line.lock().unwrap();
        if let Some((read_line, index, length)) = read_line.take() {
            if memory.get_mut(ready_index..ready_length).map(|x| x.len()) == Some(4) {
                if let Some(buf) = memory.get_mut(index..length) {
                    // Write to readline buffer
                    let capacity = buf[0..].len();
                    let length = buffer.len();
                    /*if length > capacity {
                        // Need more space!
                        
                    } else {
                        
                    }*/
                    todo!();
                    // Write to ready list
                    /*let bytes = read_line.to_le_bytes();
                    for i in 0..4 {
                        ready[i] = bytes[i];
                    }
                    return 1*/
                }
            }
        }
        0
    }
    
    fn log(&self, text: &str, level: Level, target: &str) {
        log::log!(target: target, level, "{text}")
    }
    
    unsafe fn read_line(&self, ready: u32, index: usize, length: usize) {
        let mut read_line = self.read_line.lock().unwrap();
        *read_line = Some((ready, index, length));
    }
}

fn main() -> ardaku::Result<()> {
    // Setup
    env_logger::init();

    let app_path = std::env::args().skip(1).next().expect("Provide wasm file!");
    let exe = std::fs::read(app_path).expect("Couldn't find file!");

    // Run app
    let system = System {
        buffer: Mutex::new(String::with_capacity(1024)),
        read_line: Mutex::new(None),
    };

    ardaku::run(system, &exe)
}
