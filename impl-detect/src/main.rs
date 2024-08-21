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
/// 
/// Here, more specific rules are written.
/// 1. https://rust-lang.github.io/rfcs/0195-associated-items.html#via-an-id_segment-prefix
/// 2. https://rust-lang.github.io/rfcs/0195-associated-items.html#via-a-type_segment-prefix
/// (1) tells starting with ID_SEGEMENT is equivalent to starting with TYPE_SEGMENT.
///  - 'A::b' is equivalent to '<A>::b'
/// (2) tells inherent members are priortized over in-scope traits.
pub struct ImplDetector<T>(std::marker::PhantomData<T>);

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

    #[derive(Clone)]
    struct Cloneable;
    struct UnCloneable;
    struct SendSync(i32); // i32 is both Send and Sync.
    struct SyncNotSend(std::sync::MutexGuard<'static, i32>); // MutexGuard is Sync, but not Send.
    struct SendNotSync(std::cell::Cell<i32>); // Cell is Send, but not Sync.
    struct NotSendNotSync(*mut i32); // Raw pointer is neither Send nor Sync.
    struct A;
    struct B;

    // Using syntax begining with ID, 'ID::...'
    {
        // === Detects `Clone` ===
        assert!(ImplDetector::<Cloneable>::is_clone());
        assert!(!ImplDetector::<UnCloneable>::is_clone());
        assert!(!ImplDetector::<UnCloneable>::is_clone());
        const _: () = {
            assert!(ImplDetector::<Cloneable>::IS_CLONE);
            assert!(!ImplDetector::<UnCloneable>::IS_CLONE);
        };

        // === Detects `Send` and `Sync` ===
        assert!(ImplDetector::<SendSync>::is_send());
        assert!(ImplDetector::<SendSync>::is_sync());
        assert!(!ImplDetector::<SyncNotSend>::is_send());
        assert!(ImplDetector::<SyncNotSend>::is_sync());
        assert!(ImplDetector::<SendNotSync>::is_send());
        assert!(!ImplDetector::<SendNotSync>::is_sync());
        assert!(!ImplDetector::<NotSendNotSync>::is_send());
        assert!(!ImplDetector::<NotSendNotSync>::is_sync());
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
        assert!(ImplDetector::<(A, A)>::is_equal_type());
        assert!(!ImplDetector::<(A, B)>::is_equal_type());
        const _: () = {
            assert!(ImplDetector::<(A, A)>::IS_EQUAL_TYPE);
            assert!(!ImplDetector::<(A, B)>::IS_EQUAL_TYPE);
        };
    }

    // Using syntax begining with Type, '<Type>::...'
    {
        // === Detects `Clone` ===
        assert!(<ImplDetector::<Cloneable>>::is_clone());
        assert!(!<ImplDetector::<UnCloneable>>::is_clone());
        assert!(!<ImplDetector::<UnCloneable>>::is_clone());
        const _: () = {
            assert!(<ImplDetector::<Cloneable>>::IS_CLONE);
            assert!(!<ImplDetector::<UnCloneable>>::IS_CLONE);
        };

        // === Detects `Send` and `Sync` ===
        assert!(<ImplDetector::<SendSync>>::is_send());
        assert!(<ImplDetector::<SendSync>>::is_sync());
        assert!(!<ImplDetector::<SyncNotSend>>::is_send());
        assert!(<ImplDetector::<SyncNotSend>>::is_sync());
        assert!(<ImplDetector::<SendNotSync>>::is_send());
        assert!(!<ImplDetector::<SendNotSync>>::is_sync());
        assert!(!<ImplDetector::<NotSendNotSync>>::is_send());
        assert!(!<ImplDetector::<NotSendNotSync>>::is_sync());
        const _: () = {
            assert!(<ImplDetector::<SendSync>>::IS_SEND);
            assert!(<ImplDetector::<SendSync>>::IS_SYNC);
            assert!(!<ImplDetector::<SyncNotSend>>::IS_SEND);
            assert!(<ImplDetector::<SyncNotSend>>::IS_SYNC);
            assert!(<ImplDetector::<SendNotSync>>::IS_SEND);
            assert!(!<ImplDetector::<SendNotSync>>::IS_SYNC);
            assert!(!<ImplDetector::<NotSendNotSync>>::IS_SEND);
            assert!(!<ImplDetector::<NotSendNotSync>>::IS_SYNC);
        };

        // === Detects `EqualType` ===
        assert!(<ImplDetector::<(A, A)>>::is_equal_type());
        assert!(!<ImplDetector::<(A, B)>>::is_equal_type());
        const _: () = {
            assert!(<ImplDetector::<(A, A)>>::IS_EQUAL_TYPE);
            assert!(!<ImplDetector::<(A, B)>>::IS_EQUAL_TYPE);
        };
    }
}
