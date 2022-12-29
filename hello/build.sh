export CARGO_BUILD_RUSTFLAGS="\
    --remap-path-prefix=$PWD=_ \
    --remap-path-prefix=$HOME/.local/lib/cargo=- \
    --remap-path-prefix=$HOME/.local/lib/rustup=+ \
    --remap-path-prefix=$HOME=~ \
    --remap-path-prefix=$HOME/.cargo/registry/src/=%"
cargo build --target wasm32-unknown-unknown --release && \
    cp target/wasm32-unknown-unknown/release/hello.wasm hello.wasm && \
    wasm-snip hello.wasm --snip-rust-panicking-code -o hello.wasm && \
    wasm-strip hello.wasm && \
    wasm-opt hello.wasm -o hello.wasm -Os
ls -l hello.wasm | awk '{print $5}'
