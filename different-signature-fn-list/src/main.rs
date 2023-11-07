use std::any::{TypeId, Any};
use std::collections::HashMap;

// These Data are custom structs to distinguash the same inner types.
#[derive(Debug)]
struct DataA(char);
#[derive(Debug)]
struct DataB(char);

// Concrete data storage.
// Assume that this is a storage that keeps your heterogeneous data.
// It's super simple, but for practical usage, we should make this more flexible and safe.
struct DataStorage {
    data: HashMap<TypeId, Box<dyn Any>>,
}

impl DataStorage {
    // Makes sample data.
    fn new() -> Self {
        let mut data: HashMap<TypeId, Box<dyn Any>> = HashMap::new();
        data.insert(TypeId::of::<DataA>(), Box::new(vec![DataA('a'), DataA('b')]));
        data.insert(TypeId::of::<DataB>(), Box::new(vec![DataB('c'), DataB('d')]));
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

trait Invokable {
    fn invoke(&mut self, data: &mut DataStorage); // Depends on DataPool for object safety.
}

impl<'a, T: Runnable<'a>> Invokable for T {
    #[inline]
    fn invoke(&mut self, data: &mut DataStorage) {
        self.run(
            <T::Ref as Visit>::visit(data),
            <T::Mut as VisitMut>::visit_mut(data)
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

// Please implement `Visit` and `VisitMut` for more tuples. (It's ordinary in Rust for now)
// And be careful!
// Compiler infers that lifetime of `data` is different with the `Self` because we casted to raw pointers.
// This helps we to use `data` after calling this function so that we can call `visit_mut`.
// But it's dangerous, so that we need to check borrow rule manually.
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

    // Data race occurs here on purpose.
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

    // Data race occurs here on purpose.
    fn run(&mut self, r: Self::Ref, m: Self::Mut) {
        println!("RunB");
        println!("r: {:?}", r);
        println!("m: {:?}", m);
    }
}

fn main() {
    let mut data = DataStorage::new();
    
    // We can have a list including heterogeneous functions using object safe trait `Invokable`.
    let list: Vec<Box<dyn Invokable>> = vec![Box::new(RunA), Box::new(RunB)];

    // Let's invoke each function.
    for mut item in list {
        item.invoke(&mut data);
    }
}
