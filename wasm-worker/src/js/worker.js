// This file should be shipped to the final pkg directory.
// But `wasm-bindgen` doesn't know it's needed unless binding something.
// This function is just for the binding.
export function attach() {}

onmessage = ev => {
  // This file will be located at pkg_mt/snippets/wasm-worker-?/src/js/.
  // Goes back to the same level with pkg directory,
  // then we can find JS file made by wasm-bindgen.
  import('../../../..').then((wasm) => {
    // Here's why we had to build wasm with `--target web`.
    // Wasm's default export function can import shared `memory` by doing so.
    // See https://rustwasm.github.io/wasm-bindgen/examples/raytrace.html#running-the-demo
    wasm.default(ev.data[0], ev.data[1]).then((mod) => {
      const id = ev.data[2];
      onmessage = ev => {
        mod.runWorker(ev.data, id);
      }
    })
  });
}
