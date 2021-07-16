# Syscalls
This file lists all of the syscalls available in Ardaku.  These syscalls are
necessary because WASI lacks multi-media functionality, and is not designed with
async-by-default in mind.

 - Types
   - `typedef Future: NonZero[i32]`
   - `typedef Listener: Future`
 - General (Async Fundamentals):
   - [`fn discard()`](#fn-discard)
   - [`fn listen()`](#fn-listen)
   - [`fn wait()`](#fn-wait)
 - Time
   - [`fn now()`](#fn-now)
   - [`fn timer()`](#fn-timer)


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
    (param $futures_size i32)
    (param $futures_data i32)
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

## Time

### `fn now()`
```wat
(import "ardaku" "now" (func $datetime
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
