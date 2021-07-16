(module
    (import "ardaku" "wait" (func $wait
        (param $listener i32)
        (param $futures_size i32)
        (param $futures_data i32)
        (result i32)
        (result i32)
    ))
 
    (type $t0 (func (param i32) (result i32)))

    (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
        get_local $p0 ;; comment
        i32.const 1
        i32.add
    )
)
