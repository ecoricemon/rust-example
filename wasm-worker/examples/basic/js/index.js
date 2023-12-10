// Wasm was built with `--target web`.
// So that we can put `module` and `memory` into its wbg_init function.
// In rust side, we're goint to pass them through `postMessage` to workers.
// Workers will receive and initialize wasm with them.
// As a result, workers can see the shared memory.
import wbg_init, { App } from "../pkg_mt/wasm-index.js";

// Waits for initialization of wasm.
await wbg_init();

const app = new App();
