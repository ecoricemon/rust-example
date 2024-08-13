//! Sometimes, we want to detect whether T implements traits such as [`Clone`],
//! [`Send`], [`Sync`], and so on, at run-time.
//! To do that, we can exploit rust's function lookup order.

pub trait NotClone {
    fn is_clone() -> bool { false }
}
impl<T> NotClone for T {}

pub trait NotSend {
    fn is_send() -> bool { false }
}
impl<T> NotSend for T {}

pub trait NotSync {
    fn is_sync() -> bool { false }
}
impl<T> NotSync for T {}

pub struct ImplDetector<T>(std::marker::PhantomData<T>);

impl<T: Clone> ImplDetector<T> {
    /// When someone calls [`ImplDetector::is_clone`], rust will look for 
    /// callable function in the order below
    /// - Inherent function
    /// - Trait function
    /// 
    /// So if the type is `Clone`, then rust chooses inherent function
    /// due to the search order.  
    /// But rust will choose trait function if the type is not `Clone`
    /// due to the `T: Clone` bound.
    /// 
    /// See https://doc.rust-lang.org/reference/expressions/method-call-expr.html
    /// (Document describes about methods, but I believe the same rule is applied
    /// to associated functions as well)
    pub fn is_clone() -> bool { true }
}

impl<T: Send> ImplDetector<T> {
    /// Same as above.
    pub fn is_send() -> bool { true }
}

impl<T: Sync> ImplDetector<T> {
    /// Same as above.
    pub fn is_sync() -> bool { true }
}

#[allow(dead_code)]
fn main() {
    #[derive(Clone)]
    struct Cloneable;
    assert!(ImplDetector::<Cloneable>::is_clone());

    struct UnCloneable;
    assert!(!ImplDetector::<UnCloneable>::is_clone());

    struct SendSync(i32); // i32 is both Send and Sync.
    assert!(ImplDetector::<SendSync>::is_send());
    assert!(ImplDetector::<SendSync>::is_sync());

    struct SyncNotSend(std::sync::MutexGuard<'static, i32>); // MutexGuard is Sync, but not Send.
    assert!(!ImplDetector::<SyncNotSend>::is_send());
    assert!(ImplDetector::<SyncNotSend>::is_sync());

    struct SendNotSync(std::cell::Cell<i32>); // Cell is Send, but not Sync.
    assert!(ImplDetector::<SendNotSync>::is_send());
    assert!(!ImplDetector::<SendNotSync>::is_sync());

    struct NotSendNotSync(*mut i32); // Raw pointer is neither Send nor Sync.
    assert!(!ImplDetector::<NotSendNotSync>::is_send());
    assert!(!ImplDetector::<NotSendNotSync>::is_sync());
}
