cargo build --example keyboard --target=wasm32-unknown-unknown --features web-sys
wasm-bindgen --out-dir examples/keyboard --target web --no-typescript target/wasm32-unknown-unknown/debug/examples/keyboard.wasm
