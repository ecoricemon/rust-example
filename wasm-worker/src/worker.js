const eventBuf = [];

onmessage = async ev => {
  if (typeof ev.data === 'object' && Reflect.has(ev.data, 'module')) {
    const { module, memory, import_url, init_method, id } = ev.data;

    // Imports wasm glue module.
    const wasm_glue = await import(new URL(import_url));

    // Initializes wasm with the same module and memory.
    // We use shared memory here.
    // To do that, we inserted '--target web' in our build command.
    const init = wasm_glue[init_method];
    if (init === undefined) {
      throw new Error('not found "' + init_method + '" from ' + import_url);
    }
    const wasm = await init(module, memory);

    // Consumes stacked events.
    while (eventBuf.length > 0) {
      let ev = eventBuf.shift();
      wasm.runWorker(ev.data, id);
    }

    // Run
    onmessage = ev => {
      wasm.runWorker(ev.data, id);
    }
  } else {
    // Holds events before we initialize wasm.
    eventBuf.push(ev);
  }
}
