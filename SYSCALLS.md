# Syscalls
This file lists all of the syscalls available in Ardaku.  These syscalls are
necessary because WASI lacks multi-media functionality, and is not designed with
async-by-default in mind, as well as the fact that WASI assumes a tree-style
filesystem.

 - Types
   - `typedef Future: NonZero[i32] { -> i64 }`
   - `typedef Listener: Future`

---

 - General (Async Fundamentals):
   - [`fn discard()`](#fn-discard) Discard a future.
   - [`fn listen()`](#fn-listen) Create a listener future to check when other
     futures are ready.
   - [`fn wait()`](#fn-wait) Wait on a listener future
 - General (Dynamic Data Allocation)
   - [`fn alloc()`](#fn-alloc) Dynamically gain temporary data storage from or
     cede it to the system.
 - Time
   - [`fn now()`](#fn-now) Get the current date and time.
   - [`fn timer()`](#fn-timer) Create a timer.
 - Journal - Standard I/O
   - [`fn log()`](#fn-log) Log a message to the journal.
   - [`fn prompt()`](#fn-prompt) Prompt the user to input a line of text.
 - Files (Filesystem)
   - [`fn file()`](#fn-file) Choose a file to open/load from a storage drive
     with file chooser or implicit.
   - [`fn sync()`](#fn-sync) Synchronize &amp; Save changes to storage drive if
     not already auto-saved.
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
   - [`fn buffer()`](#fn-buffer) Create a buffer on the GPU.
   - [`fn camera()`](#fn-camera) Get camera footage.
   - [`fn canvas()`](#fn-canvas) Create a canvas.
   - [`fn render()`](#fn-render) Render to a canvas.
   - [`fn screen()`](#fn-screen) Request access to a screen.
   - [`fn shader()`](#fn-shader) Load shader.
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

### `fn alloc()`
```wat
(import "ardaku" "alloc" (func $alloc
    (param $data i32)
    (param $size i32)
    (result i32)
))
```

Returns the new data index for the (re-)allocation.  Works similar to POSIX
`realloc()`.
 - If `$data` is `None` (`0`), then allocate `$size` bytes and return their
   index.
 - If `$size` is `0`, then cede the bytes at index `$data` back to the system.
 - If neither is true, resize the data (possibly changing it's index)

The new memory will not be initialized.

---

## Time

### `fn now()`
```wat
(import "ardaku" "now" (func $now
    (result i64)
))
```

Get the current date and time.  Never fails.  Returns `Date` structure:
 - `year: s16` (-32768 to 32767)
 - `month: u8` (1 to 12)
 - `day: u8` (1 to 31)
 - `hour: u8` (0 to 23)
 - `minute: u8` (0 to 60)
 - `millis: u16` (0 to 60_000 - usually, may go higher for time leaps)

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

## Journal - Standard I/O

### `fn log()`
```wat
(import "ardaku" "log" (func $log
    (param $text_data i32)
    (param $text_size i32)
))
```

Log a UTF-8 text message to the journal for debugging/investigation purposes.
You may prepend the text with a filter name followed by a null byte to allow for
easier message sorting.

### `fn prompt()`
```wat
(import "ardaku" "prompt" (func $prompt
    (param $max_size i32)
    (param $user_data i32)
    (result i32)
))
```

Prompt for a line up to `$max_size` bytes of user-inputted UTF-8 text.
If the computer runs out of memory, the text input will be limited further.
Returns a `Future` that produces a pointer to the inputted
`Text: (size: i32, data: i32)`.

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

### `fn buffer()`
**TODO** - Notes: Create and set buffer (`List[f32]`).

### `fn camera()`
**TODO** - Notes: GPU raster or CPU raster or both?

### `fn canvas()`
**TODO** - Notes: creates a GPU raster that can be rendered to.

### `fn render()`
**TODO** - Notes: render all uploaded models with transforms.

### `fn screen()`
**TODO** - Notes: request a screen, returns `None` (`0`) if all are being used.

### `fn shader()`
**TODO** - Notes: takes WebAssembly binary format with `"export"` functions -
format then converted into native code (SPIR-V, GLSL, etc.).

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
