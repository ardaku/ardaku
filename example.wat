(module
    ;; Import `event()`
    (import "ardaku" "event" (func $event
        (param $size i32)
        (param $data i32)
        (param $done i32)
        (result i32)
    ))

    ;; Export a single page memory of 64KB.
    (memory $0 (export "pages") 1)

    ;; Define constants
    (data (i32.const 1) "log")
    (data (i32.const 4) "Hello World!")
    (data (i32.const 16) "\00\01\03\01") ;; `Message` Connect to log service
    (data (i32.const 32) "\01\01\0C\04") ;; `Message` Send text to log service

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
