# The Winit Keyboard Tester

This is a program I made in order to test implementations of [#753].

It prints nicely-formatted markdown tables. You can paste those into comments
on GitHub which hopefully makes it easier to communicate about and understand
bugs and qurks in the implementation as well as the platforms.

## How to use

### Clone

Clone this repository and Winit's repository into adjacent folders.

```
parent
├── winit
└── winit_keyboard_tester
```

Alternatively, you could adjust this crate's `Cargo.toml` to conform to your
preferred folder structure.

### Build & run

#### General

For most platforms, building the program with `cargo build` should be
sufficient.

#### WASM

First, you should install `wasm-bindgen-cli` by running `cargo install
wasm-bindgen-cli`.

You can then use one of the `build-web` scripts.

### Use

When all keys have been released, the program should automatically terminate
the current table and begin a new one. The current table can also be terminated
by pressing the middle mouse button.

When the current table is empty, the middle mouse button can be used to switch
between manual and automatic mode. Manual mode is indicated in the title bar.

Indication of "manual mode" does not work on WASM as of yet.

You can reset dead key sequences on Windows and Linux by pressing the right
mouse button.

[#753]: https://github.com/rust-windowing/winit/issues/753
