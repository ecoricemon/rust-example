use std::any::{Any, TypeId};
use std::collections::HashMap;

// These custom data structures distinguish values with the same underlying type.
#[allow(dead_code)]
#[derive(Debug)]
struct DataA(char);
#[allow(dead_code)]
#[derive(Debug)]
struct DataB(char);

// Concrete data storage.
// Assume that this is a storage that keeps your heterogeneous data.
// It is deliberately simple; a production version should be more flexible and safe.
struct DataStorage {
    data: HashMap<TypeId, Box<dyn Any>>,
}

impl DataStorage {
    // Creates sample data.
    fn new() -> Self {
        let mut data: HashMap<TypeId, Box<dyn Any>> = HashMap::new();
        data.insert(
            TypeId::of::<DataA>(),
            Box::new(vec![DataA('a'), DataA('b')]),
        );
        data.insert(
            TypeId::of::<DataB>(),
            Box::new(vec![DataB('c'), DataB('d')]),
        );
        Self { data }
    }
}

trait Store {
    fn as_slice<T: 'static>(&self) -> &[T];
    fn as_mut_slice<T: 'static>(&mut self) -> &mut [T];
}

impl Store for DataStorage {
    fn as_slice<T: 'static>(&self) -> &[T] {
        self.data
            .get(&TypeId::of::<T>())
            .unwrap()
            .downcast_ref::<Vec<T>>()
            .unwrap()
            .as_slice()
    }

    fn as_mut_slice<T: 'static>(&mut self) -> &mut [T] {
        self.data
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .downcast_mut::<Vec<T>>()
            .unwrap()
            .as_mut_slice()
    }
}

trait Invocable {
    fn invoke(&mut self, data: &mut DataStorage); // Uses a concrete type for object safety.
}

impl<'a, T: Runnable<'a>> Invocable for T {
    #[inline]
    fn invoke(&mut self, data: &mut DataStorage) {
        self.run(
            <T::Ref as Visit>::visit(data),
            <T::Mut as VisitMut>::visit_mut(data),
        );
    }
}

trait Visit {
    fn visit(data: &impl Store) -> Self;
}

trait VisitMut {
    fn visit_mut(data: &mut impl Store) -> Self;
}

trait Runnable<'a> {
    type Ref: Visit;
    type Mut: VisitMut;

    fn run(&mut self, r: Self::Ref, m: Self::Mut);
}

// Implement `Visit` and `VisitMut` for additional tuple sizes as needed.
// Be careful: because we cast to raw pointers, the compiler infers that the lifetime of
// `data` is independent of `Self`. This lets us use `data` again after this function
// returns so that we can call `visit_mut`, but it requires manual borrow checking.
impl<A: 'static, B: 'static> Visit for (&[A], &[B]) {
    #[inline]
    fn visit(data: &impl Store) -> Self {
        unsafe {
            (
                &*(data.as_slice::<A>() as *const [A]),
                &*(data.as_slice::<B>() as *const [B]),
            )
        }
    }
}

impl<A: 'static, B: 'static> VisitMut for (&mut [A], &mut [B]) {
    #[inline]
    fn visit_mut(data: &mut impl Store) -> Self {
        unsafe {
            (
                &mut *(data.as_mut_slice::<A>() as *mut [A]),
                &mut *(data.as_mut_slice::<B>() as *mut [B]),
            )
        }
    }
}

struct RunA;
impl<'a> Runnable<'a> for RunA {
    type Ref = (&'a [DataA], &'a [DataB]);
    type Mut = (&'a mut [DataA], &'a mut [DataB]);

    // Aliasing occurs here intentionally for demonstration purposes.
    fn run(&mut self, r: Self::Ref, m: Self::Mut) {
        println!("RunA");
        println!("r: {:?}", r);
        println!("m: {:?}", m);
    }
}

struct RunB;
impl<'a> Runnable<'a> for RunB {
    type Ref = (&'a [DataA], &'a [DataB]);
    type Mut = (&'a mut [DataA], &'a mut [DataB]);

    // Aliasing occurs here intentionally for demonstration purposes.
    fn run(&mut self, r: Self::Ref, m: Self::Mut) {
        println!("RunB");
        println!("r: {:?}", r);
        println!("m: {:?}", m);
    }
}

fn main() {
    let mut data = DataStorage::new();

    // An object-safe `Invocable` trait lets us store heterogeneous functions in one list.
    let list: Vec<Box<dyn Invocable>> = vec![Box::new(RunA), Box::new(RunB)];

    // Let's invoke each function.
    for mut item in list {
        item.invoke(&mut data);
    }
}
