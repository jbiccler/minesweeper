# Minesweeper

This is a simple proof-of-concept Minesweeper implementation using [egui](https://github.com/emilk/egui).
It can be compiled either natively or to WebAssembly (WASM).

You can find a [demo here](https://jbiccler.github.io/minesweeper/).

## How to run it natively

To run the application natively, use the following command:

```bash
cargo run --bin minesweeper_gui
```

## How to build and run for WebAssembly (WASM)

The easiest way to build and run the WASM binary is with [Trunk](https://trunkrs.dev/).

First, make sure you have Trunk installed. If not, you can install it via cargo:

```bash
cargo install trunk
```

Second, add the wasm32-unknown-unknown build target with:

```bash
rustup target add wasm32-unknown-unknown
```

Then, to build and serve the WASM version locally, run:

```bash
cd minesweeper_gui
trunk serve --release
```

This will spawn a local server on `127.0.0.1:8081`, as configured in the `minesweeper_gui/Trunk.toml` file.
