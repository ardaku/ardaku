# Daku
Ardaku API for Rust (safe wrapper around rdaku).

## Usage
To target Ardaku, you should build with:

```bash
cargo +nightly build --target=wasm32-ardaku.json -Z build-std=std
```

Even though it is technically possible to build apps using
wasm32-unknown-unknown, there are the following benefits to using nightly:

 - Ability to use `#[cfg(target_os = "ardaku")]`
 - In the future, adding more complete standard library support.

Downsides:

 - Requires nightly, meaning things will break

The eventual goal is to get the wasm32-ardaku target built-in to Rust on stable!
