use std::sync::Mutex;

struct Wrapper<'this>(SelfReferential<'this>);

impl<'this> Wrapper<'this> {
    fn new() -> Self {
        Self(SelfReferential::new())
    }

    fn with_inner<'a, F, R>(&'a self, f: F) -> R
    where
        F: FnOnce(&'this SelfReferential<'this>) -> R
    {
        // Changes lifetime: &'a SelfReferential<'this> -> &'this SelfReferential<'this>
        let value = unsafe { std::mem::transmute(&self.0) };
        f(value)
    }
}

struct SelfReferential<'this> {
    data: String,
    refs: Mutex<Vec<&'this str>>,
}

impl<'this> SelfReferential<'this> {
    fn new() -> Self {
        Self {
            data: "hello world".to_owned(),
            refs: Mutex::new(Vec::new()),
        }
    }

    /// `self` should be borrowed as `'this` because we're going to put it in `Option<&'this str>`
    fn borrow(&'this self) {
        let mut refs = self.refs.lock().unwrap();
        refs.push(&*self.data);
    }

    /// Inserts str with `this` lifetime. Any lifetimes longer than `this` are acceptable.
    fn insert(&'this self, text: &'this str) {
        let mut refs = self.refs.lock().unwrap();
        refs.push(text);
    }

    fn print(&self) {
        let refs = self.refs.lock().unwrap();
        println!("{refs:?}");
    }
}

#[allow(unused)]
fn test_scope() {
    let scope_1 = String::from("scope 1");
    {
        let scope_2 = String::from("scope 2");
        let wrapper = Wrapper::new();
        {
            let scope_3 = String::from("scope 3");
            wrapper.with_inner(|self_ref| {
                let scope_4 = String::from("scope 4");

                self_ref.borrow();             // Stores &'wrapper
                self_ref.insert("static");     // Stores &'static
                self_ref.insert(&*scope_1);    // Stores &'scope_1
                self_ref.insert(&*scope_2);    // Stores &'scope_2
                // self_ref.insert(&*scope_3); // 'scope_3 < 'wrapper => Compile error
                // self_ref.insert(&*scope_4); // 'scope_4 < 'wrapper => Compile error
            });
        }

        wrapper.with_inner(|self_ref| {
            self_ref.print();
        });
    }
}

#[allow(unused)]
fn returning<'ret>() -> Wrapper<'ret> {
    let local = String::from("local scope");
    let wrapper = Wrapper::new();

    wrapper.with_inner(|self_ref| {
        self_ref.insert("static");   // Stores &'static
        // self_ref.insert(&*local); // any local variables < `ret => Compile error
    });

    wrapper.with_inner(|self_ref| {
        self_ref.print();
    });

    wrapper
}

fn main() {
    test_scope();
    returning();
}
