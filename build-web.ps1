cargo build --target=wasm32-unknown-unknown
wasm-bindgen --out-dir .\ --target web --no-typescript target\wasm32-unknown-unknown\debug\winit_keyboard_tester.wasm
