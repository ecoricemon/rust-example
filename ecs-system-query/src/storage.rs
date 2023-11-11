use super::{upcast_slice, Filter, QueryIter, QueryIterMut};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ptr::NonNull;

/// Our `Component`.
pub trait Component: 'static {}

/// A super simple `Component`s storage.
pub struct ComponentStorage {
    data: HashMap<TypeId, Box<dyn Any>>,
    // `query_buffer` keeps the results of queries.
    query_buffer: HashMap<(TypeId, (TypeId, TypeId)), Vec<NonNull<[()]>>>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            query_buffer: HashMap::new(),
        }
    }
    
    pub fn insert(&mut self, key: TypeId, value: Box<dyn Any>) {
        self.data.insert(key, value);
    }
}

/// Interface of the `ComponentStorage`.
/// `Store` should be able to borrow multiple internal data pieces at the same time.
/// To do that, maybe we can use interior mutability, but we use raw pointer in this example.
/// It's dangerous but easy to implement.
/// Plus, you can see the lifetimes between input and output are decoupled by explicit 'a.
pub trait Store {
    fn get<'a, F: Filter>(&mut self, q_id: (TypeId, TypeId)) -> QueryIter<'a, F::Target>;
    fn get_mut<'a, F: Filter>(&mut self, q_id: (TypeId, TypeId)) -> QueryIterMut<'a, F::Target>;
}

impl Store for ComponentStorage {
    fn get<'a, F: Filter>(&mut self, q_id: (TypeId, TypeId)) -> QueryIter<'a, F::Target> {
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

        let query_key = (TypeId::of::<F>(), q_id);
        self.query_buffer.entry(query_key)
            .and_modify(|prev| {
                // Adjust the capacity and length of the `prev` in practical usage.
                // But here we just put the new pointer in the `prev`.
                // Note that the pointer of `v` can differ from the past if it's been resized.
                prev[0] = NonNull::new(upcast_slice(v)).unwrap();
            })
            .or_insert(
                vec![NonNull::new(upcast_slice(v)).unwrap()]
            );

        // Safety: `k` is unique of all *System-Query-Filter* combinations.
        // As a result, we can guarantee that `v` is invariant during its usage because no one can generate the same `k` except itself.
        // Also, It means downcasting is valid.
        unsafe { QueryIter::new(self.query_buffer.get(&query_key).unwrap()) }
    }

    fn get_mut<'a, F: Filter>(&mut self, q_id: (TypeId, TypeId)) -> QueryIterMut<'a, F::Target> {
        let all_any_none = F::all_any_none();
        let _filters = F::as_slice(&all_any_none);

        let v = self
            .data
            .get_mut(&TypeId::of::<F::Target>())
            .unwrap()
            .downcast_mut::<Vec<F::Target>>()
            .unwrap()
            .as_mut_slice();

        let query_key = (TypeId::of::<F>(), q_id);
        self.query_buffer.entry(query_key)
            .and_modify(|prev| {
                prev[0] = NonNull::new(upcast_slice(v)).unwrap();
            })
            .or_insert(
                vec![NonNull::new(upcast_slice(v)).unwrap()]
            );

        unsafe { QueryIterMut::new(self.query_buffer.get_mut(&query_key).unwrap()) }
    }
}
