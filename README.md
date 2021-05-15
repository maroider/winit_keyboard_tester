# The Winit Keyboard Tester

This is a program I made in order to test implementations of [#753].

It prints nicely-formatted markdown tables. You can paste those into comments on GitHub which makes
it easier for everyone to communicate and understand implementation and/or platform bugs and qurks.

## How to use

### Clone

Clone this repository and Winit's repository into adjacent folders, and add the required remotes to get the
branch of the PR you want to test.

```
parent
├── winit
└── winit_keyboard_tester
```

### Build & run

#### General

For most platforms, building the program with `cargo build` should be sufficient.

#### WASM

First, you should install `wasm-bindgen-cli` by running `cargo install wasm-bindgen-cli`.

You can then use one of the `build-web` scripts.

#### Wayland

If you're using Wayland, you need to pass the program the `--enable-gl` flag, potentially like so:
`cargo run -- --enable-gl`, as windows on Wayland don't appear until they're drawn onto.

### Use

When all keys have been released, the program should automatically terminate the current table and begin a
new one. The current table can also be terminated by pressing the middle mouse button.

When the current table is empty, the middle mouse button can be used to switch between manual and automatic
mode. Manual mode is indicated in the title bar.

Indication of "manual mode" does not work on WASM as of yet.

[#753]: https://github.com/rust-windowing/winit/issues/753
