# Syscalls
This file lists all of the syscalls available in Ardaku.  These syscalls are
necessary because WASI lacks multi-media functionality, and is not designed with
async-by-default in mind, as well as the fact that WASI assumes a tree-style filesystem.

 - Types
   - `typedef Future: NonZero[i32]`
   - `typedef Listener: Future`

---

 - General (Async Fundamentals):
   - [`fn discard()`](#fn-discard) Discard a future.
   - [`fn listen()`](#fn-listen) Create a listener future to check when other futures are ready.
   - [`fn wait()`](#fn-wait) Wait on a listener future
 - General (Dynamic Data Allocation)
   - [`alloc()`](#fn-alloc) Dynamically gain temporary data storage from or cede it to the system.
 - Time
   - [`fn now()`](#fn-now) Get the current date and time.
   - [`fn timer()`](#fn-timer) Create a timer.
 - Journal - Standard I/O
   - [`fn log()`](#fn-log) Log a message to the journal.
   - [`fn prompt()`](#fn-prompt) Prompt the user to input a line of text.
 - Files (Filesystem)
   - [`fn file()`](#fn-file) Choose a file to open/load from a storage drive with file chooser or implicit.
   - [`fn sync()`](#fn-sync) Synchronize &amp; Save changes to storage drive if not already auto-saved.

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
    (result i32)
    (result i32)
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

Returns the new data index for the (re-)allocation.  Works similar to POSIX `realloc()`.
 - If `$data` is `None` (`0`), then allocate `$size` bytes and return their index.
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
    (param $user_output i32)
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

Log a UTF-8 text message to the journal for debugging/investigation purposes.  You may
prepend the text with a filter name followed by a null byte to allow for easier message
sorting.

### `fn prompt()`
```wat
(import "ardaku" "prompt" (func $prompt
    (param $max_size i32)
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
    (result i32)
))
```

Create a future that opens an existing `File` with a file chooser.  Produces
`Opt[@File]`, which is set to `None` (`0`) when the user cancels the pop-up.
If the user openned the file with this program from another program, the pop-up
will not show up.

The `File` should be closed by de-allocating the memory with `alloc(data, 0)`.

`@File: (data: i32, size: i32)`.

You may also use `alloc()` to truncate or append to the file.  Note the truncation
is destructive, unless it's to 0 bytes - then the file is closed instead of truncated.

### `fn sync()`
```wat
(import "ardaku" "sync" (func $sync
    (param $file_data i32)
    (result i32)
))
```

Synchronize file data with storage drive.

---
