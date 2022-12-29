# Ardaku
Ardaku is a general-purpose WebAssembly application engine.  It's intended to
run in any userspace program or on bare metal as sandboxing for an OS (see the
[Quantii](https://github.com/ardaku/quantii) project).

## Getting Started
To boot up Ardaku, you will need a startup application.  The hello crate is
provided in the root folder.  To compile it, run:

```bash
cd hello/
./build.sh
```

This will create a *hello.wasm* file (~10kB).  You can now run it locally with:

```bash
RUST_LOG=info cargo run --release --example demo hello/hello.wasm
```

Rust WebAssembly programs that use an allocator will always allocate at least 2
pages; this example is configured to allocate only 2:

 0. Stack (configured via rustc flags to only take up ½ page - default: 16), the
    remaining ½ page is used for the WebAssembly data section
 1. Heap allocated memory

Together this adds up to 128 kB (131\_072 bytes) for the minimum runtime memory
required by a WASM file in Ardaku.

## API
Ardaku runs the [`daku`](https://github.com/ardaku/daku) API.  You can build
your own apps for Ardaku using the `daku` crate.

## Usage
To use Ardaku on a custom target, all you need to do is implement the `System`
trait, and Ardaku takes care of the rest!

Ardaku may be used to test Quantii apps and desktop environments without running
Quantii itself.  You may also use Ardaku as an alternative to Flatpak and other
software similar to Electron (although not within a "web" context).
