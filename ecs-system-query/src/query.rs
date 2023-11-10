use super::{Component, Store};
use std::any::TypeId;

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

pub trait Identify {
    type Output;

    fn ids() -> Self::Output;
    fn as_slice(ids: &Self::Output) -> &[TypeId];
}

impl Identify for () {
    type Output = [TypeId; 0];

    #[inline]
    fn ids() -> Self::Output {
        []
    }

    #[inline]
    fn as_slice(ids: &Self::Output) -> &[TypeId] {
        ids
    }
}

impl<A: Component, B: Component> Identify for (A, B) {
    type Output = [TypeId; 2];

    #[inline]
    fn ids() -> Self::Output {
        [TypeId::of::<A>(), TypeId::of::<B>()] // Can't be used in const fn so far.
    }

    #[inline]
    fn as_slice(ids: &Self::Output) -> &[TypeId] {
        ids
    }
}

pub trait Query<'a> {
    type Output;
    type OutputMut;

    fn query(storage: &mut impl Store) -> <Self as Query<'a>>::Output;
    fn query_mut(storage: &mut impl Store) -> <Self as Query<'a>>::OutputMut;
    fn ids() -> Vec<TypeId>;
}

impl<'a, A: Filter, B: Filter> Query<'a> for (A, B) {
    type Output = (Vec<&'a [A::Target]>, Vec<&'a [B::Target]>);
    type OutputMut = (Vec<&'a mut [A::Target]>, Vec<&'a mut [B::Target]>);

    #[inline]
    fn query(storage: &mut impl Store) -> <Self as Query<'a>>::Output {
        (storage.get::<A>(), storage.get::<B>())
    }

    #[inline]
    fn query_mut(storage: &mut impl Store) -> Self::OutputMut {
        (storage.get_mut::<A>(), storage.get_mut::<B>())
    }

    #[inline]
    fn ids() -> Vec<TypeId> {
        vec![TypeId::of::<A::Target>(), TypeId::of::<B::Target>()]
    }
}
