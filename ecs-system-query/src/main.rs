//! ECS(Entity Component and System) is a pattern to optimize cache hit.
//! In this example, we're going to focus on how to implement it in terms of *System*.
//! Users can request various types of `Component`s with read and write authorities.
//! So that ECS patter should handle heterogenious types.
//! We can use Rust's associated type to solve this problem.
//! Associated type is an easy approach to show what types are passing to the *System*.

mod query;
mod system;
use query::*;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use system::*;

// Our *Component*.
pub trait Component: 'static {}

// Test *Component*s.
#[derive(Debug)]
struct CompA(&'static str);
impl Component for CompA {}
#[derive(Debug)]
struct CompB(&'static str);
impl Component for CompB {}

// A super simple *Component*s storage.
pub struct DataStorage {
    data: HashMap<TypeId, Box<dyn Any>>,
}
impl DataStorage {
    fn new() -> Self {
        let mut data: HashMap<TypeId, Box<dyn Any>> = HashMap::new();
        data.insert(
            TypeId::of::<CompA>(),
            Box::new(vec![CompA("0"), CompA("1")]),
        );
        data.insert(
            TypeId::of::<CompB>(),
            Box::new(vec![CompB("2"), CompB("3")]),
        );
        Self { data }
    }
}

// Interface of the `DataStorage`.
pub trait Store {
    // `Store` should be able to borrow multiple internal data pieces at the same time.
    // To do that, maybe we can use interior mutability, but we use raw pointer in this example.
    // It's dangerous but easy to implement.
    // Plus, you can see the lifetimes between input and output are decoupled by explicit 'a.
    fn get<'a, F: Filter>(&mut self) -> Vec<&'a [F::Target]>;
    fn get_mut<'a, F: Filter>(&mut self) -> Vec<&'a mut [F::Target]>;
}

impl Store for DataStorage {
    fn get<'a, F: Filter>(&mut self) -> Vec<&'a [F::Target]> {
        let all_any_none = F::all_any_none();
        let _filters = F::as_slice(&all_any_none);

        // Didn't implement with respect to filter.
        // Didn't check borrow rule for now, so that data race can occur.

        let v = self
            .data
            .get(&TypeId::of::<F::Target>())
            .unwrap()
            .downcast_ref::<Vec<F::Target>>()
            .unwrap()
            .as_slice();
        
        // Decouple input and output lifetimes.
        // Make sure not to write onto the data at the same time.
        let v = unsafe { &*(v as *const [F::Target]) };
        vec![v]
    }

    fn get_mut<'a, F: Filter>(&mut self) -> Vec<&'a mut [F::Target]> {
        let all_any_none = F::all_any_none();
        let _filters = F::as_slice(&all_any_none);

        // Didn't implement with respect to filter.
        // Didn't check borrow rule for now, so that data race can occur.

        let v = self
            .data
            .get_mut(&TypeId::of::<F::Target>())
            .unwrap()
            .downcast_mut::<Vec<F::Target>>()
            .unwrap()
            .as_mut_slice();

        // Decouple input and output lifetimes.
        // Make sure not to write onto the data at the same time.
        let v = unsafe { &mut *(v as *mut [F::Target]) };
        vec![v]
    }
}

// Users define their custom *System* like so,
struct SysA;
impl System for SysA {
    type Ref = (FA, FB);
    type Mut = (FA, FB);
    fn run(&self, r: <Self::Ref as Query>::Output, m: <Self::Mut as Query>::OutputMut) {
        println!("RunA");
        println!("r: {:?}", r);
        println!("m: {:?}", m);
        
        // We can see inlay type hint thanks to associated types.
        let (_a, _b) = r;
        let (_a, _b) = m;
    }
}
struct SysB;
impl System for SysB {
    type Ref = (FA, FB);
    type Mut = (FA, FB);
    fn run(&self, r: <Self::Ref as Query>::Output, m: <Self::Mut as Query>::OutputMut) {
        println!("RunB");
        println!("r: {:?}", r);
        println!("m: {:?}", m);
        let (_a, _b) = r;
        let (_a, _b) = m;
    }
}

// Users define their custom *Filter* like so.
struct FA;
impl Filter for FA {
    type Target = CompA; // Give me this component from filtered entities. 
    type FilterAll = (CompA, CompB); // Choose entities that have all these components.
    type FilterAny = (); // Choose entities that have any of these components.
    type FilterNone = (); // Don't choose entities that have any of these components.
}
struct FB;
impl Filter for FB {
    type Target = CompB;
    type FilterAll = (CompA, CompB);
    type FilterAny = ();
    type FilterNone = ();
}

fn main() {
    let mut storage = DataStorage::new();

    // We can have a list including heterogeneous functions using object safe trait `Invokable`.
    let list: Vec<Box<dyn Invokable>> = vec![Box::new(SysA), Box::new(SysB)];

    // Let's invoke each function.
    for item in list.iter() {
        item.invoke(&mut storage);
    }
}
