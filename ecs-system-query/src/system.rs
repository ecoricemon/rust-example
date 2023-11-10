use super::query::Query;
use super::DataStorage;
use std::any::TypeId;

pub trait Invokable {
    fn invoke(&self, storage: &mut DataStorage); // Depends on DataPool for object safety.
    fn reads(&self) -> Vec<TypeId>; // For parallel execution later.
    fn writes(&self) -> Vec<TypeId>; // For parallel execution later.
}

impl<T: System> Invokable for T {
    #[inline]
    fn invoke(&self, storage: &mut DataStorage) {
        let r = <T::Ref as Query>::query(storage);
        let m = <T::Mut as Query>::query_mut(storage);
        self.run(r, m);
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

pub trait System {
    type Ref: for<'a> Query<'a>;
    type Mut: for<'a> Query<'a>;

    fn run(&self, r: <Self::Ref as Query>::Output, m: <Self::Mut as Query>::OutputMut);
}
