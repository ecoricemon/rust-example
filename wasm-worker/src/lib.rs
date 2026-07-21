use once_cell::sync::OnceCell;
use wasm_bindgen::prelude::*;

/// Clients can change the initialization function from the WASM glue JavaScript before calling
/// [`Worker::spawn`]. If this value is not set, [`WBG_INIT_DEFAULT`] is used.
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
    /// Spawns a Web Worker from the current thread with the given `name` and `id`.
    /// The `name` appears in the browser's developer tools, and the job can use the `id`.
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
    /// `f` must be sendable, so it cannot contain raw pointers or `Rc` values.
    pub fn run_one_shot(&self, f: impl FnOnce(usize) + Send) -> Result<(), JsValue> {
        // Safety: `Send` is bounded by the signature.
        unsafe { self.run_one_shot_wo_send(f) }
    }

    /// Sends `f` without requiring the `Send` trait.
    /// This function is not thread-safe.
    ///
    /// # Safety
    ///
    /// `f` may access the same memory concurrently, which can cause a data race.
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
    /// Terminates the Web Worker *immediately*.
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

/// Posts JavaScript `undefined` to the parent thread that spawned the current thread.
/// See https://developer.mozilla.org/en-US/docs/Web/API/Worker/postMessage
pub fn notify_parent() {
    let global = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();

    // I believe `undefined` won't cause any errors here.
    // See https://developer.mozilla.org/en-US/docs/Web/API/Worker/postMessage
    global.post_message(&JsValue::undefined()).unwrap();
}

pub struct Job<'a> {
    /// A function to be executed by a worker.
    /// Use `Worker::run_one_shot()` to send `f` to another thread.
    /// Use `Worker::run_one_shot_wo_send()` if unrestricted access is required.
    /// Rust does not know that this is being sent to another thread, so the `Send`
    /// bound can be omitted here even though doing so is unsafe.
    f: Box<dyn 'a + FnOnce(usize)>,
}

// Some bundlers may warn about a circular dependency caused by the worker,
// such as "Rust wasm - (bind) -> worker.js -> (import) -> wasm".
// This can be avoided by removing the JavaScript file, although doing so requires additional
// bundler configuration. See the bundler's documentation for more information.
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
    /// URL of the WASM glue JavaScript file.
    //
    // Workers need this URL to import the WASM glue JavaScript dynamically and share the
    // same WASM module and memory.
    // But note that bundler may evaluate "import.meta.url" statically during bundling,
    // but we need it to be evaluated at runtime. Configure the bundler accordingly.
    // (For example, Webpack evaluates it statically by default, whereas Vite does not.)
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
