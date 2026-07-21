const eventBuf = [];

onmessage = async ev => {
  if (typeof ev.data === 'object' && Reflect.has(ev.data, 'module')) {
    const { module, memory, import_url, init_method, id } = ev.data;

    // Imports the WASM glue module.
    const wasm_glue = await import(new URL(import_url));

    // Initializes wasm with the same module and memory.
    // We use shared memory here.
    // The build command uses `--target web` to enable this.
    const init = wasm_glue[init_method];
    if (init === undefined) {
      throw new Error('not found "' + init_method + '" from ' + import_url);
    }
    const wasm = await init(module, memory);

    // Processes queued events.
    while (eventBuf.length > 0) {
      let ev = eventBuf.shift();
      wasm.runWorker(ev.data, id);
    }

    // Processes subsequent events immediately.
    onmessage = ev => {
      wasm.runWorker(ev.data, id);
    }
  } else {
    // Queues events until WASM has been initialized.
    eventBuf.push(ev);
  }
}
