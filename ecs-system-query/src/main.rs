//! ECS (Entity Component System) is a pattern that can improve cache locality.
//! This example focuses on implementing the *System* portion of an ECS.
//! Users can request read or write access to various `Component` types, so the ECS
//! must handle heterogeneous types. Rust's associated types provide a straightforward
//! way to express which types are passed to each *System*.

mod query;
mod storage;
mod system;
mod util;
use query::*;
use std::any::TypeId;
use storage::*;
use system::*;
use util::*;

// Implement `query::Identify` for tuples of various sizes.
impl_identify!(0);
impl_identify!(1, A);
impl_identify!(2, A, B);
impl_identify!(3, A, B, C);

// Implement `query::Query` for tuples of various sizes.
impl_query!(1, A);
impl_query!(2, A, B);
impl_query!(3, A, B, C);

/// Test `Component`.
#[allow(dead_code)]
#[derive(Debug)]
struct CompA(&'static str);
impl Component for CompA {}

/// Test `Component`.
#[allow(dead_code)]
#[derive(Debug)]
struct CompB(&'static str);
impl Component for CompB {}

/// Test `Filter`.
struct FA;
impl Filter for FA {
    type Target = CompA; // What you want
    type FilterAll = (CompA, CompB); // Filter used to select entities.
    type FilterAny = ();
    type FilterNone = ();
}

/// Test `Filter`.
struct FB;
impl Filter for FB {
    type Target = CompB;
    type FilterAll = (CompA, CompB);
    type FilterAny = ();
    type FilterNone = ();
}

/// Test `System`.
struct SysA;
impl System for SysA {
    type Ref = (FA, FB);
    type Mut = FA;

    // Your logic.
    fn run(&self, r: <Self::Ref as Query>::Output, m: <Self::Mut as Query>::OutputMut) {
        println!("RunA");

        // Associated types allow the editor to display useful inlay type hints.
        let (a, b) = r;
        for v in a {
            println!("r.0: {:?}", v);
        }
        for v in b {
            println!("r.1: {:?}", v);
        }
        for v in m {
            println!("m: {:?}", v);
        }
    }
}

/// Test `System`.
struct SysB;
impl System for SysB {
    type Ref = FA;
    type Mut = (FA, FB);
    fn run(&self, r: <Self::Ref as Query>::Output, m: <Self::Mut as Query>::OutputMut) {
        println!("RunB");

        // Associated types allow the editor to display useful inlay type hints.
        for v in r {
            println!("r: {:?}", v);
        }
        let (a, b) = m;
        for v in a {
            println!("m.0: {:?}", v);
        }
        for v in b {
            println!("m.1: {:?}", v);
        }
    }
}

fn main() {
    // Test storage
    let mut storage = ComponentStorage::new();
    storage.insert(
        TypeId::of::<CompA>(),
        Box::new(vec![CompA("A(0)"), CompA("A(1)")]),
    );
    storage.insert(
        TypeId::of::<CompB>(),
        Box::new(vec![CompB("B(2)"), CompB("B(3)")]),
    );

    // An object-safe `Invocable` trait lets us store heterogeneous systems in one list.
    let list: Vec<Box<dyn Invocable>> = vec![Box::new(SysA), Box::new(SysB)];

    // Let's invoke each function.
    for item in list.iter() {
        item.invoke(&mut storage);
    }
}
