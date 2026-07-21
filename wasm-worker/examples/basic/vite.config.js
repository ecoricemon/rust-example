import { defineConfig } from 'vite';
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  build: {
    rollupOptions: {
      input: {
        app: 'static/index.html',
      },
      output: {
        // The worker imports the WASM glue JavaScript to initialize WASM, but it cannot
        // access resources such as `document`. Keep the WASM glue in a separate chunk.
        manualChunks: {
          'wasm-index': ['pkg_mt/wasm-index']
        },
        // Preserve `__wbg_init` in the WASM glue JavaScript. Rollup may convert the
        // default export into a named `__wbg_init` export, so its name must remain stable.
        minifyInternalExports: false,
      }
    },
    // Relative to 'root'.
    outDir: '../dist',
  },
  // Prevents `index.html` from being emitted under `dist/static`.
  root: 'static',
  plugins: [
    // Enables top-level `await` for WASM. Alternatively, `build.target` can be set to
    // `es2022`, which also supports top-level `await`.
    wasm(),
    topLevelAwait(),
  ],
  server: {
    port: 8080,
  },
  preview: {
    port: 8080,
    // A multithreaded environment requires shared WASM memory.
    // These headers are required to share the memory.
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp'
    }
  },
});
