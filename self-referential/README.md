# A Woraround for Self-Referential Type in Rust

This example demonstrates how to work around borrow error against self-referential types in Rust.

## Motivation

Sometimes, although it's not the best option, we need to deal with types like `&'a T<'a>` for some
reason. See an example below.

```rust
struct SelfReferential<'this> {
    data: String,
    refs: Vec<&'this str>,
}

impl<'this> SelfReferential<'this> {
    fn new() -> Self {
        Self {
            data: "hello world".to_owned(),
            refs: Vec::new(),
        }
    }

    /// `self` should be borrowed as `'this` because we're going to put it in `Vec<&'this str>`
    fn borrow(&'this mut self) {
        self.refs.push(&*self.data);
    }
}

let mut self_ref = SelfReferential::new();

// Borrow lasts at the end of lifetime of `SelfReferential` itself
self_ref.borrow(); 

// We cannot borrow it again because of mutable borrow above
// self_ref.borrow();
```

## Workaround

Actually, the naive approach in the motivation section is useless. We should be able to borrow it
multiple times, so let's introduce interior mutability to our code.

```rust ignore
use std::sync::Mutex;

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

    fn borrow(&'this self) {
        let mut refs = self.refs.lock().unwrap();
        refs.push(&*self.data);
    }
}

fn make_self_referential<'a>() -> SelfReferential<'a> {
    let self_ref = SelfReferential::new();
    self_ref.borrow();
    self_ref.borrow();
    // `self_ref` is still being borrowed. We cannot return borrowed variable.
    // self_ref
}
```

We can call `borrow()` multiple times now, but we cannot return the `SelfReferential`. Ever lasting
borrow is the problem. Can we stop the borrowing before `SelfReferential` goes out of scope?

```rust
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

    fn borrow(&'this self) {
        let mut refs = self.refs.lock().unwrap();
        refs.push(&*self.data);
    }
}

fn make_self_referential<'a>() -> Wrapper<'a> {
    let wrapper = Wrapper::new();
    wrapper.with_inner(|self_ref| {
        self_ref.borrow();
        self_ref.borrow();
    });
    wrapper
}
```

By introducing a wrapper and limiting borrowing within a closure, we can now return the
`SelfReference`.  You can see more example in [`main`](src/main.rs).
