# A Workaround for Self-Referential Types in Rust

This example demonstrates how to work around borrow-checker errors involving self-referential types in Rust.

## Motivation

Sometimes, even when it is not the best option, we need to work with types such as `&'a T<'a>`.
Consider the following example.

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

    /// `self` must be borrowed for `'this` because its data will be stored in `Vec<&'this str>`.
    fn borrow(&'this mut self) {
        self.refs.push(&*self.data);
    }
}

let mut self_ref = SelfReferential::new();

// The borrow lasts until the end of the `SelfReferential` value's lifetime.
self_ref.borrow(); 

// We cannot borrow it again because of the mutable borrow above.
// self_ref.borrow();
```

## Workaround

The naive approach in the motivation section is not useful because we need to borrow the value
multiple times. We can introduce interior mutability to make that possible.

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
    // `self_ref` is still borrowed, so we cannot return it.
    // self_ref
}
```

We can now call `borrow()` multiple times, but we still cannot return the `SelfReferential`.
The problem is that the borrow never ends. Can we limit the borrow so that it ends before the
`SelfReferential` goes out of scope?

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
        // Changes the lifetime: &'a SelfReferential<'this> -> &'this SelfReferential<'this>
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
`SelfReferential`. See [`main`](src/main.rs) for a more complete example.
