//! Sometimes, we want to detect whether T implements traits such as [`Clone`],
//! [`Send`], [`Sync`], and so on, at run-time.
//! To do that, we can exploit rust's function lookup order.

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
pub struct ImplDetector<T>(std::marker::PhantomData<T>);

// NOTE: With associated functions, I tested associated constants as well.
// Although I couldn't find specification about associated constant search order,
// it works at rustc 1.80.0.
// If you'd like to do something like compile-time validation,
// constants can help you.

// === ImplDetector for `Clone` ===

pub trait NotClone {
    const IS_CLONE: bool = false;
    fn is_clone() -> bool { false }
}

impl<T> NotClone for ImplDetector<T> {}

impl<T: Clone> ImplDetector<T> {
    pub const IS_CLONE: bool = true;
    pub fn is_clone() -> bool { true }
}

// === ImplDetector for `Send` ===

pub trait NotSend {
    const IS_SEND: bool = false;
    fn is_send() -> bool { false }
}

impl<T> NotSend for ImplDetector<T> {}

impl<T: Send> ImplDetector<T> {
    pub const IS_SEND: bool = true;
    pub fn is_send() -> bool { true }
}

// === ImplDetector for `Sync` ===

pub trait NotSync {
    const IS_SYNC: bool = false;
    fn is_sync() -> bool { false }
}

impl<T> NotSync for ImplDetector<T> {}

impl<T: Sync> ImplDetector<T> {
    pub const IS_SYNC: bool = true;
    pub fn is_sync() -> bool { true }
}

// === ImplDetector for `EqualType` ===

pub trait EqualType<T> {
    const IS_EQUAL_TYPE: bool = false;
    fn is_equal_type() -> bool { false }
}

impl<T> EqualType<T> for ImplDetector<T> {}

impl<T> ImplDetector<(T, T)> {
    pub const IS_EQUAL_TYPE: bool = true;
    pub fn is_equal_type() -> bool { true }
}

#[allow(dead_code)]
fn main() {
    // === Detects `Clone` ===
    #[derive(Clone)]
    struct Cloneable;
    assert!(ImplDetector::<Cloneable>::is_clone());

    struct UnCloneable;
    assert!(!ImplDetector::<UnCloneable>::is_clone());

    // Compile-time validation.
    const _: () = {
        assert!(ImplDetector::<Cloneable>::IS_CLONE);
        assert!(!ImplDetector::<UnCloneable>::IS_CLONE);
    };

    // === Detects `Send` and `Sync` ===

    // i32 is both Send and Sync.
    struct SendSync(i32); 
    assert!(ImplDetector::<SendSync>::is_send());
    assert!(ImplDetector::<SendSync>::is_sync());

    // MutexGuard is Sync, but not Send.
    struct SyncNotSend(std::sync::MutexGuard<'static, i32>); 
    assert!(!ImplDetector::<SyncNotSend>::is_send());
    assert!(ImplDetector::<SyncNotSend>::is_sync());

    // Cell is Send, but not Sync.
    struct SendNotSync(std::cell::Cell<i32>); 
    assert!(ImplDetector::<SendNotSync>::is_send());
    assert!(!ImplDetector::<SendNotSync>::is_sync());

    // Raw pointer is neither Send nor Sync.
    struct NotSendNotSync(*mut i32); 
    assert!(!ImplDetector::<NotSendNotSync>::is_send());
    assert!(!ImplDetector::<NotSendNotSync>::is_sync());

    // Compile-time validation.
    const _: () = {
        assert!(ImplDetector::<SendSync>::IS_SEND);
        assert!(ImplDetector::<SendSync>::IS_SYNC);
        assert!(!ImplDetector::<SyncNotSend>::IS_SEND);
        assert!(ImplDetector::<SyncNotSend>::IS_SYNC);
        assert!(ImplDetector::<SendNotSync>::IS_SEND);
        assert!(!ImplDetector::<SendNotSync>::IS_SYNC);
        assert!(!ImplDetector::<NotSendNotSync>::IS_SEND);
        assert!(!ImplDetector::<NotSendNotSync>::IS_SYNC);
    };

    // === Detects `EqualType` ===
    struct A;
    struct B;

    assert!(ImplDetector::<(A, A)>::is_equal_type());
    assert!(!ImplDetector::<(A, B)>::is_equal_type());

    // Compile-time validation.
    const _: () = {
        assert!(ImplDetector::<(A, A)>::IS_EQUAL_TYPE);
        assert!(!ImplDetector::<(A, B)>::IS_EQUAL_TYPE);
    };
}
