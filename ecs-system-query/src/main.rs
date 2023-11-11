//! ECS(Entity Component and System) is a pattern to optimize cache hit.
//! In this example, we're going to focus on how to implement it in terms of *System*.
//! Users can request various types of `Component`s with read and write authorities.
//! So that ECS patter should handle heterogenious types.
//! We can use Rust's associated type to solve this problem.
//! Associated type is an easy approach to show what types are passing to the *System*.

mod query;
mod storage;
mod system;
mod util;
use query::*;
use storage::*;
use system::*;
use util::*;
use std::any::TypeId;

// impl of query::Identify for various tuples.
impl_identify!(0);
impl_identify!(1,A);
impl_identify!(2,A,B);
impl_identify!(3,A,B,C);

// impl of query::Query for various tuples.
impl_query!(1,A);
impl_query!(2,A,B);
impl_query!(3,A,B,C);

/// Test `Component`.
#[derive(Debug)]
struct CompA(&'static str);
impl Component for CompA {}

/// Test `Component`.
#[derive(Debug)]
struct CompB(&'static str);
impl Component for CompB {}

/// Test `Filter`.
struct FA;
impl Filter for FA {
    type Target = CompA; // What you want
    type FilterAll = (CompA, CompB); // Filter to select entity.
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

        // We can see inlay type hint thanks to associated types.
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

        // We can see inlay type hint thanks to associated types.
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
    storage.insert(TypeId::of::<CompA>(), Box::new(vec![CompA("A(0)"), CompA("A(1)")]));
    storage.insert(TypeId::of::<CompB>(), Box::new(vec![CompB("B(2)"), CompB("B(3)")]));

    // We can have a list including heterogeneous functions using object safe trait `Invokable`.
    let list: Vec<Box<dyn Invokable>> = vec![Box::new(SysA), Box::new(SysB)];

    // Let's invoke each function.
    for item in list.iter() {
        item.invoke(&mut storage);
    }
}
