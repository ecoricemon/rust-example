use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_worker::*;

#[allow(dead_code)]
#[wasm_bindgen]
pub struct App {
    parent: Worker,
    child_a: Rc<RefCell<Option<Worker>>>,
    child_b: Rc<RefCell<Option<Worker>>>,
    buffer: Rc<RefCell<Vec<i128>>>,
}

#[allow(clippy::new_without_default)]
#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut parent = Worker::spawn("parent", 123).unwrap();
        let child_a = Rc::new(RefCell::new(None));
        let child_b = Rc::new(RefCell::new(None));
        let buffer = Rc::new(RefCell::new(vec![0, 0, 0]));

        // Handler for main <- parent.
        {
            // Not synced, but we can see result anyway.
            let buffer = Rc::clone(&buffer);
            parent.register_callback(Closure::new(move |_| {
                let result = format!("[App] Result is {:?}", buffer.borrow());
                set_inner_html("result", &result);
            }));
        }

        // Safety:
        // - job: Intended unsafe operation
        // - parent:run_ont_shot_wo_send:
        //     `Worker::callback` looks like *mut u8, so it's thread-unsafe.
        //     But it doesn't matter because children are currently None,
        //     so that we can't access it from main thread.
        // - child...run_one_shot_wo_send():
        //      `job` has raw pointer in it, but it's on purpose.
        unsafe {
            // Can we use Arc<Mutex<T>> on wasm? I'm not sure.
            // Use raw pointers instead.
            let ptr = buffer.borrow_mut().as_mut_ptr();
            let job = move |id: usize| {
                *ptr.add(id) += 10;
                *ptr.add(2) += 10; // Two workers writes at the same memory on purpose.
            };

            // Let's make the parent spawn two children and give them jobs.
            let child_a = Rc::clone(&child_a);
            let child_b = Rc::clone(&child_b);
            let _ = parent.run_one_shot_wo_send(move |_: usize| {
                // Spawn children.
                let a = Worker::spawn("childA", 0).unwrap();
                let b = Worker::spawn("childB", 1).unwrap();
                *child_a.borrow_mut() = Some(a);
                *child_b.borrow_mut() = Some(b);

                // Runs job infinitely.
                let child_a = Rc::clone(&child_a);
                let child_b = Rc::clone(&child_b);
                let timer: Closure<dyn FnMut()> = Closure::new(move || {
                    let _ = child_a.borrow().as_ref().unwrap().run_one_shot_wo_send(job);
                    let _ = child_b.borrow().as_ref().unwrap().run_one_shot_wo_send(job);
                });
                set_interval_with_callback(timer.as_ref().unchecked_ref(), 1);

                // Leak, but it's only once.
                timer.forget();
            });
        };

        // JS will keep `App` and it won't be dropped.
        Self {
            parent,
            child_a,
            child_b,
            buffer,
        }
    }
}

// Utility
fn set_interval_with_callback(handler: &js_sys::Function, timeout: i32) {
    let global = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();
    let _ = global.set_interval_with_callback_and_timeout_and_arguments_0(handler, timeout);
}

// Utility
fn set_inner_html(id: &str, value: &str) {
    get_element_by_id(id).set_inner_html(value)
}

// Utility
fn get_element_by_id(id: &str) -> web_sys::Element {
    web_sys::window()
        .expect_throw("Failed to get window")
        .document()
        .expect_throw("Failed to get document")
        .get_element_by_id(id)
        .expect_throw("Failed to get element")
}
