use super::{upcast_slice, Filter, QueryIter, QueryIterMut};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ptr::NonNull;

/// Our `Component`.
pub trait Component: 'static {}

/// A minimal `Component` storage.
#[allow(clippy::type_complexity)]
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

/// Interface for `ComponentStorage`.
/// A `Store` must be able to borrow multiple internal values at the same time.
/// Interior mutability is one option, but this example uses raw pointers because they
/// are straightforward to demonstrate. This is dangerous. The explicit `'a` also
/// decouples the input and output lifetimes.
pub trait Store {
    fn get<'a, F: Filter>(&mut self, q_id: (TypeId, TypeId)) -> QueryIter<'a, F::Target>;
    fn get_mut<'a, F: Filter>(&mut self, q_id: (TypeId, TypeId)) -> QueryIterMut<'a, F::Target>;
}

impl Store for ComponentStorage {
    fn get<'a, F: Filter>(&mut self, q_id: (TypeId, TypeId)) -> QueryIter<'a, F::Target> {
        let all_any_none = F::all_any_none();
        let _filters = F::as_slice(&all_any_none);

        // Filtering is not implemented in this example.
        // Borrowing rules are not checked, so aliasing or data races may occur.

        let v = self
            .data
            .get_mut(&TypeId::of::<F::Target>())
            .unwrap()
            .downcast_mut::<Vec<F::Target>>()
            .unwrap()
            .as_mut_slice();

        let query_key = (TypeId::of::<F>(), q_id);
        self.query_buffer
            .entry(query_key)
            .and_modify(|prev| {
                // A production implementation should adjust the capacity and length of `prev`.
                // Here, we only replace its pointer. Note that resizing `v` may change its pointer.
                prev[0] = NonNull::new(upcast_slice(v)).unwrap();
            })
            .or_insert(vec![NonNull::new(upcast_slice(v)).unwrap()]);

        // Safety: `k` is unique among all *System-Query-Filter* combinations.
        // Therefore, `v` remains unchanged while in use because no other combination can
        // generate the same `k`. This also makes the downcast valid.
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
        self.query_buffer
            .entry(query_key)
            .and_modify(|prev| {
                prev[0] = NonNull::new(upcast_slice(v)).unwrap();
            })
            .or_insert(vec![NonNull::new(upcast_slice(v)).unwrap()]);

        unsafe { QueryIterMut::new(self.query_buffer.get_mut(&query_key).unwrap()) }
    }
}
