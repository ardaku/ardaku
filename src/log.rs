use devout::{log, Tag};
use flume::{Receiver, Sender};
use std::cell::Cell;
use wasmer::{LazyInit, Memory, MemoryView, WasmerEnv};

// How many log messages can be written at once without blocking.
const CONFIG_CONCURRENT_LOGS: usize = 2;

const LOG: Tag = Tag::new("Log").show(true);

/// Messages the program sends to the driver.
enum Cmd {
    /// Send message to the logger.
    Log(String),
}

/// Messages the driver sends to the program.
enum Msg {
    /// Logger is ready to accept new messages.
    Ready(String),
}

#[derive(WasmerEnv, Clone)]
pub struct State {
    // Command Sender
    cmd: Sender<Cmd>,
    // Message Receiver
    msg: Receiver<Msg>,

    // Memory exported by the wasm module
    #[wasmer(export(name = "pages"))]
    memory: LazyInit<Memory>,
}

impl State {
    /// Start the logging thread.
    pub(super) fn new() -> State {
        let (cmd, cmd_receiver) = flume::bounded::<Cmd>(CONFIG_CONCURRENT_LOGS);
        let (msg_sender, msg) = flume::bounded::<Msg>(CONFIG_CONCURRENT_LOGS);
        let memory = wasmer::LazyInit::new();

        std::thread::spawn(move || start(cmd_receiver, msg_sender));

        State { cmd, msg, memory }
    }

    /// The exported syscall
    pub(super) fn log(&self, data: i32, size: i32) {
        let memory = self.memory.get_ref().unwrap();
        let view: MemoryView<u8> = memory.view();
        let mut view_iter = view[data as usize..(data + size) as usize]
            .iter()
            .map(Cell::get)
            .peekable();
        let mut c = [0u8; 4];

        while view_iter.peek().is_some() {
            // Request a buffer.
            let Msg::Ready(mut buffer) = self.msg.recv().unwrap();

            'l: for _ in 0..80 {
                'c: for i in 0..4 {
                    c[i] = if let Some(ch) = view_iter.next() {
                        ch
                    } else {
                        break 'l;
                    };
                    match std::str::from_utf8(&c[..i + 1]) {
                        Ok(c) => {
                            buffer.push_str(if c == "\0" { "�" } else { c });
                            break 'c;
                        }
                        Err(_e) if _e.error_len().is_some() => {
                            buffer.push(char::REPLACEMENT_CHARACTER);
                            break 'c;
                        }
                        _ => continue,
                    }
                }
            }

            self.cmd.send(Cmd::Log(buffer)).unwrap();
        }
    }
}

fn start(receiver: Receiver<Cmd>, sender: Sender<Msg>) {
    // Offer up configured number string buffers for concurrent logging.
    for _ in 0..CONFIG_CONCURRENT_LOGS {
        sender.send(Msg::Ready(String::with_capacity(128))).unwrap();
    }

    // Wait for log commands.
    for cmd in receiver.iter() {
        match cmd {
            Cmd::Log(mut text) => {
                log!(LOG, "{}", text);
                text.clear();
                sender.send(Msg::Ready(text)).unwrap();
            }
        }
    }
}
