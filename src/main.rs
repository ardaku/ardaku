use wasmer::{
    imports, Exports, Function, Instance, LazyInit, Memory, MemoryType, Module,
    Store, Value, WasmerEnv, Universal, Cranelift
};

#[derive(WasmerEnv, Clone)]
pub struct State {
    #[wasmer(export(name = "shared"))]
    memory: LazyInit<Memory>,
}

impl State {
    fn log(&self, data: i32, size: i32) {
        let memory = self.memory.get_ref().unwrap();
        // unsafe: This is safe because the memory can't be modified anywhere
        // else - it belongs to this thread.
        let text = String::from_utf8_lossy(unsafe {
            &memory.data_unchecked()[data as usize..(data + size) as usize]
        });
        println!("{}", text);
    }
}

fn main() {
    let binary = include_bytes!("../example.wasm");

    let store = Store::new(&Universal::new(Cranelift::new()).engine());
    let module = Module::from_binary(&store, &binary[..]).unwrap();
    let state = State {
        memory: LazyInit::new(),
    };

    // The module doesn't import anything, so we create an empty import object.
    let import_object = imports! {
        "ardaku" => {
            "log" => Function::new_native_with_env(&store, state, State::log),
        }
    };
    let instance = Instance::new(&module, &import_object).unwrap();

    // Start the app.
    let start = instance.exports.get_function("start").unwrap();
    start.call(&[]).unwrap();
}
