/// Logging API.
mod log;

use log::State as Log;
use pasts::Loop;
use wasmer::{
    imports, Cranelift, Function, Instance, Module, Store, Universal,
};

// Shared state between tasks on the main (wasm) thread.
struct State;

async fn run() {
    let mut state = State;

    Loop::<_, (), _>::new(&mut state).await;
}

fn main() {
    let binary = include_bytes!("../example.wasm");

    let store = Store::new(&Universal::new(Cranelift::new()).engine());
    let module = Module::from_binary(&store, &binary[..]).unwrap();
    let log = Log::new();

    // The module doesn't import anything, so we create an empty import object.
    let import_object = imports! {
        "ardaku" => {
            "log" => Function::new_native_with_env(&store, log, Log::log),
        }
    };
    let instance = Instance::new(&module, &import_object).unwrap();

    // Start the app.
    let start = instance.exports.get_function("start").unwrap();
    start.call(&[]).unwrap();

    pasts::block_on(run())
}
