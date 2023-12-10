export function createWorker(name) {
  // `memory` must be a WebAssembly.Memory with shared flag.
  // This is required to post `memory` to workers.
  // See https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer
  if (!crossOriginIsolated) {
    throw new Error("Not crossOriginIsolated");
  }

  // Webpack requires import.meta.url.
  // See https://webpack.js.org/guides/web-workers/
  let worker = new Worker(new URL('./worker.js', import.meta.url), {
    type: 'module',
    name,
  });

  return worker;
}
