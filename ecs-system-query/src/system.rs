use super::query::Query;
use super::ComponentStorage;
use std::any::TypeId;

#[allow(dead_code)]
pub trait Invocable {
    fn invoke(&self, storage: &mut ComponentStorage); // Uses a concrete type for object safety.
    fn reads(&self) -> Vec<TypeId>; // For parallel execution later.
    fn writes(&self) -> Vec<TypeId>; // For parallel execution later.
}

impl<T: System> Invocable for T {
    #[inline]
    fn invoke(&self, storage: &mut ComponentStorage) {
        self.run(
            <T::Ref as Query>::query(storage, TypeId::of::<T>()),
            <T::Mut as Query>::query_mut(storage, TypeId::of::<T>()),
        );
    }

    #[inline]
    fn reads(&self) -> Vec<TypeId> {
        <T::Ref as Query>::ids()
    }

    #[inline]
    fn writes(&self) -> Vec<TypeId> {
        <T::Mut as Query>::ids()
    }
}

pub trait System: 'static {
    type Ref: for<'a> Query<'a>;
    type Mut: for<'a> Query<'a>;

    fn run(&self, r: <Self::Ref as Query>::Output, m: <Self::Mut as Query>::OutputMut);
}
