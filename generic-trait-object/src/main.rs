//! # Trait object from trait with generic methods
//!
//! ## Situation
//!
//! - We want to create a trait object from a trait that has generic methods.
//! - The generic methods require a `'static` lifetime, as in `foo<T: 'static>()`.
//!
//! ## Pattern abstraction
//!
//! In Rust, only object-safe traits can become trait objects, and generic methods are not object-safe.
//! To overcome this limitation, non-generic methods can accept `dyn Any` parameters and then call
//! the generic methods. We can inspect the `TypeId` of a `dyn Any`, but a `TypeId` does not reveal
//! its concrete type. We therefore inject functions that call the generic methods with concrete
//! types, then invoke the appropriate function for each `TypeId`.
//! This approach is unsafe, extremely verbose, and inefficient, but it demonstrates one solution.
//!
//! ## Reference
//!
//! https://github.com/dtolnay/erased-serde/blob/master/explanation/main.rs

use core::mem::{swap, zeroed};
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    fmt::Debug,
};

/// Trait bounds for the generic methods.
/// The `'static` bound is required to use `dyn Any`.
trait Element: 'static + Debug {}

/// Our target.
trait Generic {
    fn generic_writes<E: Element>(&mut self, param: &mut E);
    fn generic_reads<E: Element>(&mut self, param: &mut E);
    fn foo(&self) -> &'static str;
}

/// The type-erased counterpart of `Generic`, used to create a trait object.
trait ErasedGeneric {
    fn erased_writes(&mut self, param: &mut dyn Any);
    fn erased_reads(&mut self, param: &mut dyn Any);
    fn erased_foo(&self) -> &'static str;
}

/// An implementation of `Generic` and `ErasedGeneric`.
struct Handler {
    fn_table: HandlerFnTable,
    v: Vec<Box<dyn Any>>, // Anonymous Vec
}

/// Exposes generic methods through a trait object.
/// This is the first call on the call stack.
impl Generic for dyn ErasedGeneric {
    #[inline]
    fn generic_writes<E: Element>(&mut self, param: &mut E) {
        self.erased_writes(param as &mut dyn Any);
    }

    #[inline]
    fn generic_reads<E: Element>(&mut self, param: &mut E) {
        self.erased_reads(param as &mut dyn Any);
    }

    #[inline]
    fn foo(&self) -> &'static str {
        self.erased_foo()
    }
}

/// This is the second call on the call stack.
impl ErasedGeneric for Handler {
    #[inline]
    fn erased_writes(&mut self, param: &mut dyn Any) {
        let ty_id = (param as &dyn Any).type_id();
        let fn_table = self
            .fn_table
            .generic_writes
            .take()
            .expect("fn_table_writes must be filled.");
        let delegator = fn_table
            .get(&ty_id)
            .expect("fn_table_writes doesn't have appropriate entry.");
        (delegator)(self, param);
        self.fn_table.generic_writes = Some(fn_table); // Gives it back.
    }

    #[inline]
    fn erased_reads(&mut self, param: &mut dyn Any) {
        let ty_id = (param as &dyn Any).type_id();
        let fn_table = self
            .fn_table
            .generic_reads
            .take()
            .expect("fn_table_reads must be filled.");
        let delegator = fn_table
            .get(&ty_id)
            .expect("fn_table_reads doesn't have appropriate entry.");
        (delegator)(self, param);
        self.fn_table.generic_reads = Some(fn_table);
    }

    #[inline]
    fn erased_foo(&self) -> &'static str {
        // Doesn't require `FnTable`.
        self.foo()
    }
}

/// Real implementations for the trait `Generic`.
/// This is the third and final call on the call stack.
impl Generic for Handler {
    fn generic_writes<E: Element>(&mut self, param: &mut E) {
        // Simple and unsafe writing test.
        // Make sure to keep your code as safe as possible.
        let mut dummy: E = unsafe { zeroed() };
        swap(&mut dummy, param);
        let input = dummy;
        self.v.push(Box::new(input));

        println!("generic_writes() got an object of {:?}", TypeId::of::<E>());
    }

    fn generic_reads<E: Element>(&mut self, param: &mut E) {
        // Same as above.
        if !self.fn_table.types.contains(&TypeId::of::<E>()) {
            self.fn_table.add::<E>();
        }

        // Following reading test.
        let mut elem = self.v.pop().expect("There are no elements on the stack.");
        if let Some(casted) = elem.downcast_mut::<E>() {
            swap(casted, param);
        }

        println!("generic_reads() got an object of {:?}", TypeId::of::<E>());
    }

    fn foo(&self) -> &'static str {
        "Handler::foo()"
    }
}

/// This is a literal function table.
/// We can call a specific function using the `TypeId` of a `dyn Any`.
/// Each function in this table calls the real generic method.
type FnTable = HashMap<TypeId, Box<dyn Fn(&mut Handler, &mut dyn Any)>>;

/// `FnTable`s for Handler.
struct HandlerFnTable {
    // The tables are temporarily taken by `impl ErasedGeneric`, so they are stored as `Option`s.
    generic_writes: Option<FnTable>,
    generic_reads: Option<FnTable>,

    // Used only for quick membership checks.
    types: HashSet<TypeId>,
}

/// Provides an integrated builder for `FnTable`s.
/// This implementation is only one possible approach. You can ignore the builder and add
/// entries to an `FnTable` wherever appropriate. See `add()` for an example.
impl HandlerFnTable {
    // Empty tables.
    fn new() -> Self {
        Self {
            generic_writes: Some(FnTable::new()),
            generic_reads: Some(FnTable::new()),
            types: HashSet::new(),
        }
    }

    // Allows entries to be chained after `new()`.
    #[allow(dead_code)]
    fn with<T: Element>(mut self) -> Self {
        self.add::<T>();
        self
    }

    // Inserts new entry.
    fn add<T: Element>(&mut self) -> &mut Self {
        if let Some(map) = self.generic_writes.as_mut() {
            map.insert(
                TypeId::of::<T>(),
                Box::new(|handler: &mut Handler, value: &mut dyn Any| {
                    handler.generic_writes(value.downcast_mut::<T>().unwrap());
                }),
            );
        }
        if let Some(map) = self.generic_reads.as_mut() {
            map.insert(
                TypeId::of::<T>(),
                Box::new(|handler: &mut Handler, value: &mut dyn Any| {
                    handler.generic_reads(value.downcast_mut::<T>().unwrap());
                }),
            );
        }
        self.types.insert(TypeId::of::<T>());
        self
    }
}

/// Test type A
#[derive(Debug, PartialEq)]
struct A {
    _a1: u8,
    _a2: u8,
}

/// Test type B
#[derive(Debug, PartialEq)]
struct B {
    _b1: u16,
}

/// 'static and other bounds.
impl Element for A {}
impl Element for B {}

fn main() {
    // Constructs a trait object.
    let mut handler = Handler {
        fn_table: HandlerFnTable::new(),
        v: Vec::new(),
    };
    handler.fn_table.add::<A>().add::<B>();
    let mut trait_object: Box<dyn ErasedGeneric> = Box::new(handler);

    // Writes something in order to test the trait object.
    trait_object.generic_writes(&mut A { _a1: 1, _a2: 2 });
    trait_object.generic_writes(&mut B { _b1: 3 });

    // Reads back.
    let mut a_read = A { _a1: 0, _a2: 0 };
    let mut b_read = B { _b1: 0 };
    trait_object.generic_reads(&mut b_read);
    trait_object.generic_reads(&mut a_read);

    // Works as expected?
    assert_eq!(A { _a1: 1, _a2: 2 }, a_read);
    assert_eq!(B { _b1: 3 }, b_read);

    // The printed types should match the types used in the generic methods.
    println!("Type A's id: {:?}", TypeId::of::<A>());
    println!("Type B's id: {:?}", TypeId::of::<B>());

    // Non-generic method is also callable on the trait object.
    println!("{}", trait_object.foo());
}
