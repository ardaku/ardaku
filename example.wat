(module
    ;; Import `event()`
    ;; (import "ardaku" "event" (func $event
    ;;     (param $size i32)
    ;;     (param $data i32)
    ;;     (param $done i32)
    ;;     (result i32)
    ;; ))

    ;; Export a single page memory of 64KB.
    (memory $0 (export "pages") 1)

    ;; Define constants
    (data (i32.const 1) "log")
    (data (i32.const 4) "Hello World!")

    ;; Declare the `start()` function
    (func $start (export "start")
        ;; Create new logging service channel at index 1.
        ;; (call $event (i32.const 0) (i32.const 1) (i32.const 3) (i32.const 1))
        ;; Send message over logging service channel.
        ;; (call $event (i32.const 1) (i32.const 1) (i32.const 12) (i32.const 4))
    )
)
