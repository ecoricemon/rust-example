use super::util::{downcast_slice, downcast_mut_slice};
use super::{Component, Store};
use std::slice::{Iter, IterMut};
use std::{any::TypeId, marker::PhantomData};
use std::ptr::NonNull;

/// A filter to select slices of `Component`.
/// Users should fill this form of filter.
/// `Target` is what `Component` you want. You will receive slices of this `Target`.
/// `FilterAll` is a tuple of `Component`s to choose entities that have all these `Component`s.
/// `FilterAny` is a tuple of `Component`s to choose entities that have any of these `Component`s.
/// `FilterNone` is a tuple of `Component`s not to choose entities that have any of these `Component`s.
pub trait Filter: 'static {
    type Target: Component;
    type FilterAll: Identify;
    type FilterAny: Identify;
    type FilterNone: Identify;

    #[allow(clippy::type_complexity)]
    #[inline]
    fn all_any_none() -> (
        <Self::FilterAll as Identify>::Output,
        <Self::FilterAny as Identify>::Output,
        <Self::FilterNone as Identify>::Output,
    ) {
        (
            <Self::FilterAll as Identify>::ids(),
            <Self::FilterAny as Identify>::ids(),
            <Self::FilterNone as Identify>::ids(),
        )
    }

    #[allow(clippy::type_complexity)]
    #[inline]
    fn as_slice(
        all_any_none: &(
            <Self::FilterAll as Identify>::Output,
            <Self::FilterAny as Identify>::Output,
            <Self::FilterNone as Identify>::Output,
        ),
    ) -> [&[TypeId]; 3] {
        [
            <Self::FilterAll as Identify>::as_slice(&all_any_none.0),
            <Self::FilterAny as Identify>::as_slice(&all_any_none.1),
            <Self::FilterNone as Identify>::as_slice(&all_any_none.2),
        ]
    }
}

/// A trait to get `TypeId`s of elements inside a tuple.
pub trait Identify {
    type Output;

    fn ids() -> Self::Output;
    fn as_slice(ids: &Self::Output) -> &[TypeId];
}

pub trait Query<'a> {
    type Output;
    type OutputMut;

    fn query(storage: &mut impl Store, s_id: TypeId) -> Self::Output;
    fn query_mut(storage: &mut impl Store, s_id: TypeId) -> Self::OutputMut;
    fn ids() -> Vec<TypeId>;
}

pub struct QueryMutTypeIdSalt;

pub struct QueryIter<'a, T> {
    iter: Iter<'a, NonNull<[()]>>,
    _marker: PhantomData<T>,
}

impl<'a, T> QueryIter<'a, T> {
    /// # Safety
    /// 
    /// Borrow check breaks here.
    /// Caller should guarantee that `v` is invariant during its usage.
    /// Plus, generic parameter `T` should match with the original type of the `v`.
    pub unsafe fn new(v: &Vec<NonNull<[()]>>) -> Self {
        Self {
            iter: (*(v as *const Vec<NonNull<[()]>>)).iter(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for QueryIter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|ptr| 
                // Safety: Downcasting will be guaranteed by the caller(See comment at the constructor).
                unsafe { downcast_slice(ptr.as_ptr()) }
            )
    }
}

pub struct QueryIterMut<'a, T> {
    iter: IterMut<'a, NonNull<[()]>>,
    _marker: PhantomData<T>,
}

impl<'a, T> QueryIterMut<'a, T> {
    pub unsafe fn new(v: &mut Vec<NonNull<[()]>>) -> Self {
        Self {
            iter: (*(v as *mut Vec<NonNull<[()]>>)).iter_mut(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for QueryIterMut<'a, T> {
    type Item = &'a mut [T];

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|ptr| unsafe { downcast_mut_slice(ptr.as_ptr()) })
    }
}
