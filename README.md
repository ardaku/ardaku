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

This will create a *hello.wasm* file.  You can now run it locally with:

```bash
RUST_LOG=info cargo run --release --example demo hello/hello.wasm
```

## API
Ardaku runs the [`daku`](https://github.com/ardaku/daku) API.  You can build
your own apps for Ardaku using the `daku` crate.

## Usage
To use Ardaku on a custom target, all you need to do is implement the `System`
trait, and Ardaku takes care of the rest!

Ardaku may be used to test Quantii apps and desktop environments without running
Quantii itself.  You may also use Ardaku as an alternative to Flatpak and other
software similar to Electron (although not within a "web" context).
