# Ardaku
Ardaku is a general-purpose WebAssembly application engine.  It's intended to
run in any userspace program or on bare metal as sandboxing for an OS (see the
[Arc](https://github.com/ardaku/arc) project).

## Trait
To use Ardaku as a library, you must implement a trait:

```rust
pub trait Syscall {
    /// Read UTF-32 character from stdin.
    fn read(&self) -> u32;

    /// Write UTF-32 character to stdout.
    fn write(&self, byte: u32);

    /// Return kernel version
    fn version(&self) -> u32;

    /// Reboot the system.
    fn reboot(&self);
}
```

## API
The API for applications to communicate with Ardaku and each other is based on
channel communication inspired by kqueue (reduced syscalls) and IOCP
(completion-based).  The API consists of a single function:

```wat
(import "ardaku" "event" (func $event
    (param $size i32)   ;; Message list size
    (param $data i32)   ;; Message list reference
    (param $done i32)   ;; Reference to channels ready (size=open channel count)
    (result i32)        ;; Number of channels ready (max 16384 - size trunc.)
))
```

This function sends messages on channels in the message list, then waits until
at least one message is ready, then writes the channel IDs into `$done`, and
returns the size.  Then, usually the channel IDs are used to index and execute
functions in an array of function references.  The message struct looks like
this:

```rust
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
```

Disconnecting from service 0 with empty buffer stops the program.
Disconnecting from service with buffer is reserved for future use (crashes).

When `channel_id` is 0, service connector service - by registered name:

 - `log`: Receives UTF-8 log messages and saves them (stdout)
 - `prompt`: Sends UTF-8 debug commands (stdin)

 - `screen`: Receives pixels to display on the screen
 - `camera`: Sends pixels from "a source" / Receives settings commands

 - `speakers`: Receives f32 interleaved audio to play
 - `microphone`: Sends f32 interleaved audio to record

 - `save`: Receives atomic file patches
 - `load`: Sends requested sections of a file

 - `share`: Receives file for exporting to other app
 - `grab`: Sends file from other app for importing

 - `haptic`: Receives haptic events
 - [`input`](input.md): Sends input events / Receives input events for subprocesses

You can disconnect a channel in the middle of it's processing as a way of
cancelling the I/O.  Usually, if not always the I/O will complete anyway - it
just won't notify you (file I/O cancelling might be added in the future).

If the channel message has not been "completed", then the program is expected
to not try sending another message until it receives a message back from the
service.  If a message is sent before the other one has been processed, the
program will crash.  It's up to the programmer to choose and implement one of:
buffering, dropping, cancelling, or blocking of messages.

 - buffering: build a bounded (or unbounded) queue of messages - bounded queues
   must fall back to dropping, cancelling or blocking.
 - dropping: ignore the most recent event.
 - cancelling: cancel and replace the last message with most recent event.
 - blocking: send message immediately upon completion of last message.

Which method is used depends on the type of program being written, choose
wisely!

## Getting Started
To boot up Ardaku, you will need a startup application.  The file *example.wat*
is provided in the root folder.  To compile it, you will need to install wabt.
Once you do, run:

```bash
wat2wasm example.wat
```

This will create an *example.wasm* file.  You can now run it locally with:

```bash
cargo run --release example.wasm
```

## Ideas
 - The operating system should be able to run as an application within another
   operating system (avoiding the need for VMs)
 - Operating systems should be designed with a security-first mindset, making
   all programs sandboxed by default (making WebAssembly a good target)
 - Programs compiled for the operating system should be able to be
   re-distributed without having to match CPU architecture, while also running
   at native speeds (which Wasmer can handle pretty well)
 - Syscalls should be simple, high level mathematical functions that make it
   difficult to make programming errors, while also being powerful and fast
 - Adding support for new platforms should be as easy as implementing a trait of
   all the syscalls.
