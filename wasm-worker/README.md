# Web Workers + WASM + Vite Example

## Explanation

This example shows how to use Web Workers from Rust while bundling all JavaScript and WASM assets.
It demonstrates how to spawn workers and assign jobs that operate on shared memory. All workers
share the same WASM memory, so they can read and write the same memory locations concurrently.

## Prerequisites

### Rust nightly toolchain

- This example uses the `nightly-2024-06-20` Rust toolchain.
- You can adapt it to another version, but the behavior may differ.
- Install the nightly toolchain with `rustup`:
  ```sh
  rustup install nightly-2024-06-20
  ```
- When you build the project, you may see an error like this:
  ```sh
  error: "/your/home/.rustup/toolchains/nightly-2024-06-20-???/lib/rustlib/src/rust/Cargo.lock" does not exist, unable to build with the standard library,
  try:
    rustup component add rust-src --toolchain nightly-2024-06-20-???
  ```
- Follow the suggestion in the error message. The target triple (represented by `???`) varies by host machine.

## References

> [wasm-bindgen example](https://github.com/rustwasm/wasm-bindgen/tree/main/examples/raytrace-parallel)
