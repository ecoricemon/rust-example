# Example of Web Worker + WASM + Webpack

## Explanation

If you're trying to use Web Worker in your Rust and you want to bundle all of your
JS and wasm, this example may be able to help you.
This shows how to spawn web workers and give them jobs especially on shared memory.
All workers shares the same wasm memory, so that they can read/write same memory location simultaneously.

## Prerequisites

### Rust nightly toolchain
- This example is based on rust toolchain nightly-2023-12-07, which is the latest now.
- You can adapt different version, but it might work differently.
- You can install nightly toolchain using rustup.
  ```sh
  rustup install nightly-2023-12-07
  ```
- But you'll see an error like below when you're trying to build,
  ```sh
  error: "/your/home/.rustup/toolchains/nightly-2023-12-07-???/lib/rustlib/src/rust/Cargo.lock" does not exist, unable to build with the standard library,
  try:
    rustup component add rust-src --toolchain nightly-2023-12-07-???
  ```
- Nice advice. Let's follow the instruction. Notice that the target triple(represented by ???) will differ from host machine to machine.

## References

> [wasm-bindgen example](https://github.com/rustwasm/wasm-bindgen/tree/main/examples/raytrace-parallel)
