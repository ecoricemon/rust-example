use wasm_bindgen::prelude::*;

/// Binds JS.
#[wasm_bindgen(module = "/src/js/workerGen.js")]
extern "C" {
    /// Spawn new worker in JS side in order to let bundler know
    #[wasm_bindgen(js_name = "createWorker")]
    fn create_worker(name: &str) -> web_sys::Worker;
}

/// Binds JS.
#[wasm_bindgen(module = "/src/js/worker.js")]
extern "C" {
    #[wasm_bindgen]
    fn attach();
}

impl Drop for Worker {
    /// Terminates web worker *immediately*.
    fn drop(&mut self) {
        self.handle.terminate();
        log!("Worker({}) was terminated", &self.name);
    }
}

#[derive(Debug)]
pub struct Worker {
    handle: web_sys::Worker,
    name: String,
    callback: Closure<dyn FnMut(web_sys::Event)>,
}

impl Worker {
    /// Spawns a web worker from current thread with the given `name` and `id`.
    /// You can see `name` in browser's dev tool.
    /// And you can use `id` in your job.
    pub fn spawn(name: &str, id: usize) -> Result<Self, JsValue> {
        // Creates a new worker.
        let handle = create_worker(name);

        // Sets default callback.
        let callback = Closure::new(|_| notify_parent());
        handle.set_onmessage(Some(callback.as_ref().unchecked_ref()));

        // Initializes the worker.
        let message = js_sys::Array::new_with_length(3);
        message.set(0, wasm_bindgen::module());
        message.set(1, wasm_bindgen::memory());
        message.set(2, id.into());
        handle.post_message(&message)?;

        Ok(Self {
            handle,
            name: name.to_owned(),
            callback,
        })
    }

    /// Registers `callback`.
    /// `callback` will be invoked when `run_one_shot` has finished.
    pub fn register_callback(&mut self, callback: Closure<dyn FnMut(web_sys::Event)>) {
        self.handle
            .set_onmessage(Some(callback.as_ref().unchecked_ref()));
        self.callback = callback;
    }

    /// Requests to run `f` only once.
    /// `f` should be sendable, it means `f` can't have raw pointer or Rc inside it.
    pub fn run_one_shot(&self, f: impl FnOnce(usize) + Send) -> Result<(), JsValue> {
        // Safety: `Send` is bounded by the signature.
        unsafe { self.run_one_shot_wo_send(f) }
    }

    /// You can send `f` without `Send` trait.
    /// But this function is not thread-safe.
    ///
    /// # Safety
    ///
    /// `f` can access the same memory simultaneously, so that race can occur.
    #[inline]
    pub unsafe fn run_one_shot_wo_send(&self, f: impl FnOnce(usize)) -> Result<(), JsValue> {
        // Packs `f` with Box.
        // Can we remove Box here?
        let job = Box::new(Job { f: Box::new(f) });

        // Extracts raw pointer from the `job`.
        // Worker threads will release the memory.
        let job_ptr = Box::into_raw(job);

        // Sends `job_ptr` to the worker.
        self.handle.post_message(&JsValue::from(job_ptr))
    }
}

/// Entry point called by JS worker threads.
/// You may be able to use `worker_id` in your job closure if you want to.
///
/// # Safety
///
/// `job_ptr` should be valid.
#[wasm_bindgen(js_name = "runWorker")]
pub unsafe fn run_worker(job_ptr: *mut Job, worker_id: usize) {
    let job = unsafe { Box::from_raw(job_ptr) };
    (job.f)(worker_id);
    notify_parent();
}

/// Post JS `undefined` to the parent thread which spawned current thread.
/// See https://developer.mozilla.org/en-US/docs/Web/API/Worker/postMessage
pub fn notify_parent() {
    let global = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();

    // I believe `undefined` won't cause any errors here.
    // See https://developer.mozilla.org/en-US/docs/Web/API/Worker/postMessage
    global.post_message(&JsValue::undefined()).unwrap();
}

pub struct Job<'a> {
    /// A function worker will do.
    /// Use `Worker::run_one_shot()` to send `f` to other threads.
    /// You can use `Worker::run_one_shot_wo_send()` if you need unlimited access.
    /// Note that Rust doesn't know we're sending this to other threads,
    /// So that we can omit `Send` bound here even if it's unsafe.
    f: Box<dyn 'a + FnOnce(usize)>,
}

pub mod util {
    #[macro_export]
    macro_rules! log {
        ($($t:tt)*) => {
            $crate::util::console_log(format!($($t)*));
        }
    }

    pub fn console_log(s: String) {
        web_sys::console::log_1(&s.into());
    }

    pub fn hardware_concurrency() -> Option<usize> {
        Some(web_sys::window()?.navigator().hardware_concurrency() as usize)
    }
}
