use once_cell::sync::OnceCell;
use wasm_bindgen::prelude::*;

/// Clients can modify init function of wasm glue JS file before they call [`Worker::spawn`].
/// If you don't set this value, [`WBG_INIT_DEFAULT`] will be set as default value.
///
/// # Example
///
/// ```rust
/// use wasm_worker::*;
///
/// // Some bundlers may minify export name to '_'.
/// crate::WBG_INIT.set("_".to_owned()).unwrap();
/// let worker = Worker::spawn("worker", 0).unwrap();
/// ```
pub static WBG_INIT: OnceCell<String> = OnceCell::new();

pub const WBG_INIT_DEFAULT: &str = "__wbg_init";

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
        let handle = create_worker(name)?;

        // Sets default callback.
        let callback = Closure::new(|_| notify_parent());
        handle.set_onmessage(Some(callback.as_ref().unchecked_ref()));

        // Sets 'WBG_INIT' if it wasn't set yet.
        let init_method = WBG_INIT.get_or_init(|| WBG_INIT_DEFAULT.to_owned());

        // Initializes the worker.
        use js_sys::{Object, Reflect};
        let msg = Object::new();
        Reflect::set(&msg, &"module".into(), &wasm_bindgen::module())?;
        Reflect::set(&msg, &"memory".into(), &wasm_bindgen::memory())?;
        Reflect::set(&msg, &"import_url".into(), &IMPORT_META_URL.as_str().into())?;
        Reflect::set(&msg, &"init_method".into(), &init_method.into())?;
        Reflect::set(&msg, &"id".into(), &id.into())?;
        handle.post_message(&msg)?;

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

impl Drop for Worker {
    /// Terminates web worker *immediately*.
    fn drop(&mut self) {
        self.handle.terminate();
        log!("Worker({}) was terminated", &self.name);
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

// Some bundlers could warn about circular dependency caused by worker
// such as "Rust wasm - (bind) -> worker.js -> (import) -> wasm".
// We can avoid it by removing JS file although it requires other types of settings to bundler.
// See bundler's configuration for more information.
fn create_worker(name: &str) -> Result<web_sys::Worker, JsValue> {
    web_sys::Worker::new_with_options(
        &script_url(),
        web_sys::WorkerOptions::new()
            .name(name)
            .type_(web_sys::WorkerType::Module),
    )
}

fn script_url() -> String {
    let js = include_str!("worker.js");
    let blob_parts = js_sys::Array::new_with_length(1);
    blob_parts.set(0, JsValue::from_str(js));

    let mut options = web_sys::BlobPropertyBag::new();
    options.type_("application/javascript");

    let blob = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &options).unwrap();
    web_sys::Url::create_object_url_with_blob(&blob).unwrap()
}

#[wasm_bindgen]
extern "C" {
    /// URL of wasm glue JS file.
    //
    // We need this URL of wasm glue JS file in order to import it dynamically in workers.
    // So that workers can share the same wasm module and memory.
    // But note that bundler may evaluate "import.meta.url" statically during bundling,
    // which is not what we want, we need to evaluate it at runtime.
    // Therefore, you need to configure your bundler not to do it.
    // (e.g. Webpack does it basically, But Vite doesn't do it)
    #[wasm_bindgen(js_namespace = ["import", "meta"], js_name = url)]
    static IMPORT_META_URL: String;
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
