#[inline]
pub fn upcast_slice<T>(v: &mut [T]) -> *mut [()] {
    v as *mut [T] as *mut [()]
}

#[inline]
pub unsafe fn downcast_slice<'a, T>(ptr: *mut [()]) -> &'a [T] {
    &*(ptr as *const [T])
}

#[inline]
pub unsafe fn downcast_mut_slice<'a, T>(ptr: *mut [()]) -> &'a mut [T] {
    &mut *(ptr as *mut [T])
}

#[macro_export]
macro_rules! impl_identify {
    (0) => {
        impl $crate::query::Identify for () {
            type Output = [std::any::TypeId; 0];
        
            #[inline]
            fn ids() -> Self::Output {
                []
            }
        
            #[inline]
            fn as_slice(ids: &Self::Output) -> &[std::any::TypeId] {
                ids
            }
        }
    };
    (1, $id:ident) => {
        impl<$id: $crate::storage::Component> $crate::query::Identify for $id {
            type Output = [std::any::TypeId; 1];
        
            #[inline]
            fn ids() -> Self::Output {
                [std::any::TypeId::of::<$id>()]
            }
        
            #[inline]
            fn as_slice(ids: &Self::Output) -> &[std::any::TypeId] {
                ids
            }
        }
    };
    ($n:tt, $($id:ident),+) => {
        impl<$($id: $crate::storage::Component),+> $crate::query::Identify for ( $($id),+ ) {
            type Output = [std::any::TypeId; $n];
        
            #[inline]
            fn ids() -> Self::Output {
                [$(std::any::TypeId::of::<$id>()),+]
            }
        
            #[inline]
            fn as_slice(ids: &Self::Output) -> &[std::any::TypeId] {
                ids
            }
        }
    }
}

#[macro_export]
macro_rules! impl_query {
    (1, $id:ident) => {
        impl<'a, $id: $crate::query::Filter> $crate::query::Query<'a> for $id {
            type Output = $crate::query::QueryIter<'a, $id::Target>;
            type OutputMut = $crate::query::QueryIterMut<'a, $id::Target>;
            
            #[inline]
            fn query(storage: &mut impl $crate::storage::Store, s_id: std::any::TypeId) -> Self::Output {
                storage.get::<$id>((std::any::TypeId::of::<Self>(), s_id))
            }

            #[inline]
            fn query_mut(storage: &mut impl $crate::storage::Store, s_id: std::any::TypeId) -> Self::OutputMut {
                storage.get_mut::<$id>((std::any::TypeId::of::<Self>(), s_id))
            }
            
            #[inline]
            fn ids() -> std::vec::Vec<std::any::TypeId> {
                vec![std::any::TypeId::of::<$id::Target>()]
            }
        }
    };
    ($n:tt, $($id:ident),+) => {
        impl<'a, $($id: $crate::query::Filter),+> $crate::query::Query<'a> for ( $($id),+ ) {
            type Output = ( $($crate::query::QueryIter<'a, $id::Target>),+ );
            type OutputMut = ( $($crate::query::QueryIterMut<'a, $id::Target>),+ );
            
            #[inline]
            fn query(storage: &mut impl $crate::storage::Store, s_id: std::any::TypeId) -> Self::Output {
                ( 
                    $( storage.get::<$id>((std::any::TypeId::of::<Self>(), s_id)) ),+
                )
            }

            #[inline]
            fn query_mut(storage: &mut impl $crate::storage::Store, s_id: std::any::TypeId) -> Self::OutputMut {
                ( 
                    $(
                        storage.get_mut::<$id>((
                        std::any::TypeId::of::<(Self, $crate::query::QueryMutTypeIdSalt)>(), 
                        s_id)) 
                    ),+
                )
            }
            
            #[inline]
            fn ids() -> std::vec::Vec<std::any::TypeId> {
                vec![$(std::any::TypeId::of::<$id::Target>()),+]
            }
        }
    }
}
