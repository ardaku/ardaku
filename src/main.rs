use wasmer::{Store, Module, Instance, Value, imports};

fn main() {
    let binary = include_bytes!("../example.wasm");

    let store = Store::default();
    let module = Module::from_binary(&store, &binary[..]).unwrap();
    // The module doesn't import anything, so we create an empty import object.
    let import_object = imports! {};
    let instance = Instance::new(&module, &import_object).unwrap();

    let add_one = instance.exports.get_function("add_one").unwrap();
    let result = add_one.call(&[Value::I32(43)]).unwrap();
    assert_eq!(result[0], Value::I32(44));
}
