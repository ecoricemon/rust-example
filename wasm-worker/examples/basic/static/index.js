// Wasm was built with `--target web`.
// This lets us pass `module` and `memory` to its `wbg_init` function.
// On the Rust side, we pass them to workers through `postMessage`.
// The workers receive these values and use them to initialize WASM.
// As a result, workers can see the shared memory.
import wbg_init, { App } from "../pkg_mt/wasm-index.js";

// Waits for WASM to initialize.
await wbg_init();

const app = new App();
