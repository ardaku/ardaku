(module
    ;; Import Syscalls 
    (import "ardaku" "log" (func $log
        (param $text_data i32)
        (param $text_size i32)
    ))

    ;; Export a single page memory of 64KB.
    (memory $0 (export "pages") 1)

    ;; Define constants
    (data (i32.const 0) "Hello World!")

    ;; Declare the `start()` function
    (func $start (export "start")
        (call $log (i32.const 0) (i32.const 12))
    )
)
