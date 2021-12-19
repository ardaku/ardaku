(module
    ;; Import `event()`
    (import "ardaku" "event" (func $event
        (param $size i32)
        (param $data i32)
        (param $done i32)
        (result i32)
    ))

    ;; Export a single page memory of 64KB.
    (memory $0 (export "memory") 1)

    ;; Define constants
    (data (i32.const 1) "log")
    (data (i32.const 4) "Hello World!")
    ;; `Message` Connect to log service
    (data (i32.const 16) "\00\00\00\00") ;; Connect
    (data (i32.const 20) "\01\00\00\00") ;; Index 1
    (data (i32.const 24) "\03\00\00\00") ;; Name length 3
    (data (i32.const 28) "\01\00\00\00") ;; Pointer to "log" @1
    ;; `Message` Send text to log service
    (data (i32.const 32) "\01\00\00\00") ;; Index 1: Log
    (data (i32.const 36) "\01\00\00\00") ;; Set Index 1 (don't change it)
    (data (i32.const 40) "\0C\00\00\00") ;; Message length 12
    (data (i32.const 44) "\04\00\00\00") ;; Pointer to "Hello World!" @4

    ;; Set the start function
    (start $start)

    ;; Declare the `start()` function
    (func $start
        ;; Create new logging service channel at index 1.
        (call $event (i32.const 1) (i32.const 16) (i32.const 48))
        (drop) ;; Ignore return value
        ;; Send message over logging service channel.
        (call $event (i32.const 1) (i32.const 32) (i32.const 48))
        (drop) ;; Ignore return value
    )
)
