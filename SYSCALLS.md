# Syscalls
This file lists all of the syscalls available in Ardaku.  These syscalls are
necessary because WASI lacks multi-media functionality, and is not designed with
async-by-default in mind, as well as the fact that WASI assumes a tree-style
filesystem.

Allocation is not handled by the syscalls.  Neither is memory-mapped file
support.  Both must be implemented in the WebAssembly module.  It's encouraged
to reduce allocation as much as possible, and use simpler allocators (such as
bump) with maximum data sizes.  The minimum number of pages is 256 in wasmer,
and it's expected to try to stay around that number if possible.

 - Types
   - `typedef Future: NonZero[i32] { -> i64 }`
   - `typedef Listener: Future`

---

 1. [`fn connector()`](#fn-connector) - Check for new devices to connect to
 2. [`fn timer()`](#fn-timer) - Set up a CPU timer
 3. [`fn now()`](#fn-now) - Get the time from the OS
 4. [`fn log()`](#fn-log) - Write out to the debug log
 5. [`fn discard()`](#fn-discard) - Discard future (relinquish resources)
 6. [`fn watch()`](#fn-watch) - Add event(s) to watcher, and wait for event
 7. [`fn open()`](#fn-open) - Request to open a different file (Ctrl-E)
 8. [`fn load()`](#fn-load) - Load a page from the file
 9. [`fn save()`](#fn-save) - Save a page to the file
 10. [`fn share()`](#fn-share) - Share current file with another app (Alt-S)
 11. [`fn request()`](#fn-request) - Request a file from another app (Alt-E)
 12. [`fn prompt()`](#fn-prompt) - Prompt the user (debug log)

---

## `fn connector()`
```wat
(import "ardaku" "connector" (func $connector
    (param $hardware_id i32)
    (result i32)
))
```

Create a connector for a specific type of hardware.

 - 0x0000_0000: MIDI Device
 - 0x0000_0001: Speakers (Audio Playback Device)
 - 0x0000_0002: Microphone (Audio Capture Device)
 - 0x0000_0003: Camera (Webcam)
 - 0x0000_0004: Screen (Display, Monitor)
 - 0x0000_0005: Gamepad (Joystick)
 - 0x0000_0006: Flightstick
 - 0x0000_0007: USB Input (Other)
 - 0x0000_0008: Keyboard
 - 0x0000_0009: Mouse
 - 0x0000_000A: RESERVED
 - ...........
 - 0xFFFF_FFFF: RESERVED

## `fn timer()`
```wat
(import "ardaku" "timer" (func $timer
    (param $nanos i64)
    (result i32)
))
```

Create a timer Future.

## `fn now()`
```wat
(import "ardaku" "now" (func $now
    (result i32)
))
```

Queue a task that produces the current date and time.  Never fails.  Produces
`@Date i64` structure:
 - `year: s16` (-32768 to 32767)
 - `month: u8` (1 to 12)
 - `day: u8` (1 to 31)
 - `hour: u8` (0 to 23)
 - `minute: u8` (0 to 60)
 - `millis: u16` (0 to 60_000 - usually, may go higher for time leaps)

## `fn log()`
```wat
(import "ardaku" "log" (func $now
    (param $text i32)
    (param $size i32)
    (result i32)
))
```

Log a UTF-8 text message to the journal for debugging/investigation purposes.
You may prepend the text with a filter name followed by a null byte to allow for
easier message sorting.  Returns future that is ready when the queued task
writes out to the system log.  Always returns the same future, so it's not
necessary to discard it.

## `fn discard()`
```wat
(import "ardaku" "discard" (func $discard
    (param $future i32)
))
```

Relinquish resources associated with a future.

## `fn watch()`
```wat
(import "ardaku" "watch" (func $watch
    (param $watcher i32) ;; The watcher to watch for events from
    (param $events i32)  ;; Opt[@($count i32, [($fut i32, $fn i32); $count])]
    (result i64)         ;; Return value from event reactor
))
```

The `watch()` syscall does a blocking watch for asynchronous events, and reacts
to them.  It can be called in several ways:

 - `(call $watch (param 0) (param $context))`: Returns a new empty watcher.
 - `(call $watch (param $watcher) (param 0))`: Watch for events on an existing
   watcher, and return the value from the event reactor.
 - `(call $watch (param $watcher) (param $events))`: Add events to the watcher,
   then watch for them, and return the value from the event reactor.
 - `(call $watch (param $future) (param 0))`: Watch a single future, and return
   the value it produces.  Currently the second parameter is ignored.

Events stop being watched when [`discard()`](#fn-discard) is called on the
Future.  Usually, the return value from a future will be `@_` (a reference).
These references will become invalid once `watch()` is called again.

The function pointer should follow this type:

```wat
(func $reactor
    (param $context i32)  ;; The user context passed at creation
    (param $produced i64) ;; Value produced by the future
    (result i64)          ;; Return value from event reactor
)
```

If you've created a watcher and don't want it to immediately watch, you must add
it as an Future to an existing watcher.  Blocking calls to `watch` will then
return -1.

## `fn open()`
```wat
(import "ardaku" "open" (func $open
    (result i32) ;; Returns future
))

(import "ardaku" "storage" (memory $storage 1))
```

This function opens a new file.  The program doesn't know the filename or any
associated tags (there are no file paths in Ardaku).  The future produces an
address and file future if the file has opened, and 0 if no file was opened.
The address is memory-mapped from the disk.  Files are limited to ~256TB.  You
can have one web-assembly page open for the file at a time.

## `fn load()`
```wat
(import "ardaku" "load" (func $load
    (m_page i32) ;; Memory Page index [0:2^32-1]
    (f_page i32) ;; File Page index [0:2^32-1]
    (result i32) ;; Returns future
))
```

Load file page by index.  File pages are the same size as web assembly pages
(64KB).  This makes it possible to address files up to 256TB on a 32-bit CPU.
If the file needs to become larger, it does so automatically.

## `fn save()`
```wat
(import "ardaku" "save" (func $save
    (m_page i32) ;; Memory Page index [0:2^32-1]
    (f_page i32) ;; File Page index [0:2^32-1]
    (result i32) ;; Returns future
))
```

Save file page by index.

## `fn prompt()`
```wat
(import "ardaku" "prompt" (func $prompt
    (param $max_size i32)
    (param $user_data i32)
    (result i32)
))
```

Prompt for a line up to `$max_size` bytes of user-inputted UTF-8 text.
If the computer runs out of memory, the text input will be limited further.
Returns a `Future` that produces the inputted `Text: i64(size: i32, data: i32)`.

## `fn share()`
```wat
(import "ardaku" "share" (func $share
    (result i32)
))
```

Share the current file with another application.

## `fn request()`
```wat
(import "ardaku" "request" (func $request
    (result i32)
))
```

Get future to request a file data stream from another application.  The future
produces a `DataStream` future and the size of the file in pages.

---



 - Cryptographically-Secure Random Number Generator
   - [`fn rand()`](#fn-rand) Generate a random bit pattern.
 - Task Spawning
   - [`fn task()`](#fn-task) Start a task on the thread-pool.
 - General Purpose I/O
   - [`fn gpio()`](#fn-gpio) Set input and output for each port.
   - [`fn read()`](#fn-read) Read an analogue GPIO signal.
 - Networking
   - [`fn disconnect()`](#fn-close) Close socket connection.
   - [`fn recv()`](#fn-recv) Receive network packet(s).
   - [`fn send()`](#fn-send) Send network packet(s).
   - [`fn connect()`](#fn-socket) Open socket for communication (bluetooth, TCP,
     UDP, I2C, etc.).
 - Graphics
   - [`fn camera()`](#fn-camera) Get camera footage.
   - [`fn canvas()`](#fn-canvas) Create a canvas.
   - [`fn render()`](#fn-render) Render to a canvas.
   - [`fn screen()`](#fn-screen) Request access to a screen.
   - [`fn model()`](#fn-model) Load GLB model file.
 - Audio
   - [`fn speakers()`](#fn-speakers) Query and select speakers
   - [`fn microphone()`](#fn-microphone) Query and select microphone
   - [`fn play()`](#fn-play) Play audio
   - [`fn record()`](#fn-record) Record audio
 - Human Interface Devices (Input - Feedback)
   - FIXME: Keyboard (Keys)
   - FIXME: Keyboard (Text)
   - FIXME: Mouse / Touchscreen / Touchpad / Cursor
   - FIXME: Gamepad
   - FIXME: Joystick
   - FIXME: Other HID
 - System Info
   - [`fn info()`](#fn-info) Get CPU info
 - Sensors
   - FIXME: Gyro
   - FIXME: Battery %
   - FIXME: Accelerometer
   - FIXME: GPS

---

## General (Async Fundamentals)

### `fn discard()`
```wat
(import "ardaku" "discard" (func $discard
    (param $future i32)
))
```

Discard a `Future` when it's no longer needed.  If provided `Future` is invalid,
Ardaku will abort the application - so it's important to verify your program's
correctness when using this syscall in order to avoid crashing.

### `fn listen()`
```wat
(import "ardaku" "listen" (func $listen
    (param $user_data i32)
    (result i32)
))
```

Create and a new `Listener` future.  The listener future produces the output of
each future as they become ready.  This function returns `Opt[Listener]`, and
only returns `None` (`0`) if out of memory.

### `fn wait()`
```wat
(import "ardaku" "wait" (func $wait
    (param $listener i32)
    (param $futures_data i32)
    (param $futures_size i32)
    (result i64) ;; Future output
    (result i32) ;; Associated user data
))
```

Wait for a future to be ready.  Returns the data associated with the future.
Optionally, may add to the list of polled futures.  If there are no futures,
waits indefinitely.  `Future`s may be used on multiple `Listener`s.  `Future`s
are removed from all `Listener`s when they are discarded.  If provided
`Listener` is invalid, Ardaku will abort the application - so it's important to
verify your program's correctness when using this syscall in order to avoid
crashing.

---

## General (Dynamic Data Allocation)


---

## Time

### `fn now()`
```wat
(import "ardaku" "now" (func $now
    (result i64)
))
```



### `fn timer()`
```wat
(import "ardaku" "timer" (func $timer
    (param $duration i64)
    (param $user_data i32)
    (result i32)
))
```

Create a new timer `Future` (`Timer`).  The future outputs the number of times
the timer went off since the previous call to `wait()` that polled this future.
Duration is 64-bit in nanoseconds (meaning over 580 year timers are supported,
as 10-second timers would not be possible with 32-bit duration).  This function
returns `Opt[Timer]`, and only returns `None` (`0`) if out of memory.

---



---

## Files (Filesystem)

### `fn file()`
```wat
(import "ardaku" "file" (func $file
    (param $user_data i32)
    (result i32)
))
```

Create a future that opens an existing `File` with a file chooser.  Produces
`Opt[@File]`, which is set to `None` (`0`) when the user cancels the pop-up.
If the user openned the file with this program from another program, the pop-up
will not show up.

The `File` should be closed by de-allocating the memory with `alloc(data, 0)`.

`@File: (data: i32, size: i32)`.

You may also use `alloc()` to truncate or append to the file.  Note the
truncation is destructive, unless it's to 0 bytes - then the file is closed
instead of truncated.

### `fn sync()`
```wat
(import "ardaku" "sync" (func $sync
    (param $file_data i32)
    (param $user_data i32)
    (result i32)
))
```

Synchronize file data with storage drive.  Returns a `Future`, that becomes
ready when the synchronization is complete.

---

## Cryptographically-Secure Random Number Generator

### `fn rand()`
```wat
(import "ardaku" "rand" (func $rand
    (param $output_data i32)
    (param $output_size i32)
    (result i32)
))
```

Generate cryptographically secure random byte patterns.  Returns a `Future`,
that becomes ready once the random number generation is complete.

---

## Task Spawning

### `fn task()`
```wat
(import "ardaku" "task" (func $task
    (param $task_func i32)
    (param $task_param i32)
    (param $priority i32)
    (param $name i32) ;; @Text
    (param $user_data i32)
    (result i32)
))
```

Start a task on the thread-pool.  Returns a `Future` that becomes ready once the
task has finished.

---

## General Purpose I/O

### `fn gpio()`
```wat
(import "ardaku" "gpio" (func $gpio
    (param $is_output i64)
    (result i32)
))
```

Returns a reference to 64-bit array of in/out pins, of which modes are set via
the `$is_output` bits.

### `fn read()`
```wat
(import "ardaku" "read" (func $read
    (param $analogue_pin_id i32)
    (param $user_data i32)
    (result i32)
))
```

Read a fixed-point analogue signal.  Returns a `Future { -> i32 }` as the
conversion may take time (and have it's own specialized hardware).

---

## Networking

### `fn close()`
**TODO**

### `fn recv()`
**TODO**

### `fn send()`
**TODO**

### `fn socket()`
**TODO**

---

## Graphics

### `fn camera()`
**TODO** - Notes: GPU raster or CPU raster or both?

### `fn canvas()`
**TODO** - Notes: creates a GPU raster that can be rendered to.

### `fn render()`
**TODO** - Notes: render all uploaded models with transforms.

### `fn screen()`
**TODO** - Notes: request a screen, returns `None` (`0`) if all are being used.

### `fn model()`
```wat
(import "ardaku" "model" (func $model
    (param $glb_data i32)
    (param $glb_size i32)
    (result i32)
))
```

Load the binary format for GLTF to the GPU.  Returns a `Future { -> i32 }, that
becomes ready with the model ID once loaded.  The GLB file contains PBR
(physically based rendering inputs to the shader) and vertex data.

---

## Audio

### `fn speakers()`
**TODO**

### `fn microphone()`
**TODO**

### `fn play()`
**TODO** - Notes: Returns `Future { -> @[f32]}` to audio buffer.

### `fn record()`
**TODO** - Notes: Returns `Future { -> @[f32]}` to audio buffer.

---

## Human Interface Devices (Input - Feedback)



---

## System Info

### `fn read()`
```wat
(import "ardaku" "info" (func $info
    (param $which i32)
    (result i32)
))
```

Get System info
 - 0x00\_00\_00\_00: Number of CPUs (1-)
 - 0x00\_00\_00\_01: Number of GPUs (0-)
 - 0x00\_00\_00\_10: Number of GPIO Pins (0-)
 - 0x00\_00\_00\_11: Number of Analog GPIO Pins (0-)
 - 0x00\_00\_00\_12: Number of Digital GPIO Pins (0-)

---

## Sensors

---
