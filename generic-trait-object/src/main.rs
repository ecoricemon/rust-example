//! # Trait object from trait with generic methods
//!
//! ## Situation
//!
//! - Want to make a trait obejct from a trait having some generic methods.
//! - Generic methods require 'static lifetime such as `foo<T: 'static>()`.
//!
//! ## Pattern abstration
//!
//! In Rust, only object safe traits can become trait objects, generic methods make them not object safe.
//! To overcome this limitation, we can use `dyn Any` as parameters to take generic arguments
//! from non-generic methods, and call the generic methods in them.
//! We can inspect `TypeId`s from the 'dyn Ayn's, but we can't know concrete types from the 'TypeId's.
//! So we're going to inject functions calling generic methods with the concrete types,
//! and invoke those functions according to the `TypeId`s.
//! It's unsafe, extremely verbose and inefficient but I couldn't come up with any other solutions.
//!
//! ## Reference
//!
//! https://github.com/dtolnay/erased-serde/blob/master/explanation/main.rs

use core::mem::{swap, zeroed};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

/// Trait bounds for the generic methods.
/// 'static must be included to use `dyn Any`.
trait Element: 'static + Debug {}

/// Our target.
trait Generic {
    fn generic_writes<E: Element>(&mut self, param: &mut E);
    fn generic_reads<E: Element>(&mut self, param: &mut E);
    fn foo(&self) -> &'static str;
}

/// Generic erased.
/// We're going to make a trait object based on this.
trait ErasedGeneric {
    fn erased_writes(&mut self, param: &mut dyn Any);
    fn erased_reads(&mut self, param: &mut dyn Any);
    fn erased_foo(&self) -> &'static str;
}

/// This is a literally function table.
/// We can call a specific funtion using `TypeId` from the `dyn Any`.
/// Each function in this table calls the real generic method.
type FnTable = HashMap<TypeId, Box<dyn Fn(&mut Handler, &mut dyn Any)>>;

/// An implementation of `Generic` and `ErasedGeneric`.
struct Handler {
    // Tables will be taken in `impl ErasedGeneric` so that they are type of `Option`.
    fn_table_writes: Option<FnTable>,
    fn_table_reads: Option<FnTable>,
    v: Vec<Box<dyn Any>>, // Anonymous Vec
}

/// impl for exposure of generic methods from trait object.
/// This is the first call on call stack.
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

/// This is the second call on call stack.
impl ErasedGeneric for Handler {
    #[inline]
    fn erased_writes(&mut self, param: &mut dyn Any) {
        let ty_id = (param as &dyn Any).type_id();
        let fn_table = self
            .fn_table_writes
            .take()
            .expect("fn_table_writes must be filled.");
        let delegator = fn_table
            .get(&ty_id)
            .expect("fn_table_writes doesn't have appropriate entry.");
        (delegator)(self, param);
        self.fn_table_writes = Some(fn_table); // Gives it back.
    }

    #[inline]
    fn erased_reads(&mut self, param: &mut dyn Any) {
        let ty_id = (param as &dyn Any).type_id();
        let fn_table = self
            .fn_table_reads
            .take()
            .expect("fn_table_reads must be filled.");
        let delegator = fn_table
            .get(&ty_id)
            .expect("fn_table_reads doesn't have appropriate entry.");
        (delegator)(self, param);
        self.fn_table_reads = Some(fn_table);
    }
    
    #[inline]
    fn erased_foo(&self) -> &'static str {
        // Doesn't require `FnTable`.
        self.foo()
    }
}

/// Real implementations for the trait `Generic`.
/// This is the third and final call on call stack.
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
        // Following reading test.
        let mut elem = self.v.pop().expect("There's no elements stacked.");
        if let Some(casted) = elem.downcast_mut::<E>() {
            swap(casted, param);
        }

        println!("generic_reads() got an object of {:?}", TypeId::of::<E>());
    }
    
    fn foo(&self) -> &'static str {
        "Handler::foo()"
    }
}

/// Helper macro to insert an entry into `FnTable`.
macro_rules! add_delegator {
    ($fn_table:ident, $handler:ty, $generic_ty:ty, $generic_fn:ident) => {
        $fn_table.insert(
            std::any::TypeId::of::<$generic_ty>(),
            Box::new(|handler: &mut $handler, value: &mut dyn Any| {
                handler.$generic_fn(value.downcast_mut::<$generic_ty>().unwrap());
            }),
        );
    };
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
    // `fn_table` has delegators to call the real generic methods.
    let mut fn_table_writes = FnTable::new();
    let mut fn_table_reads = FnTable::new();
    add_delegator!(fn_table_writes, Handler, A, generic_writes);
    add_delegator!(fn_table_writes, Handler, B, generic_writes);
    add_delegator!(fn_table_reads, Handler, A, generic_reads);
    add_delegator!(fn_table_reads, Handler, B, generic_reads);

    // Constructs a trait object.
    let handler = Handler {
        fn_table_writes: Some(fn_table_writes),
        fn_table_reads: Some(fn_table_reads),
        v: Vec::new(),
    };
    let mut trait_object: Box<dyn ErasedGeneric> = Box::new(handler);

    // Writes something.
    trait_object.generic_writes(&mut A { _a1: 1, _a2: 2 });
    trait_object.generic_writes(&mut B { _b1: 3 });

    // Reads back.
    let mut a_read = A { _a1: 0, _a2: 0 };
    let mut b_read = B { _b1: 0 };
    trait_object.generic_reads(&mut b_read);
    trait_object.generic_reads(&mut a_read);
    assert_eq!(A { _a1: 1, _a2: 2 }, a_read);
    assert_eq!(B { _b1: 3 }, b_read);

    println!("Type A's id: {:?}", TypeId::of::<A>());
    println!("Type B's id: {:?}", TypeId::of::<B>());
    
    // Calls non-generic method.
    println!("{}", trait_object.foo());
}
