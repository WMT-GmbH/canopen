use crate::objectdictionary::datalink::DataLink;
use core::fmt;

#[inline]
pub const fn metadata(ptr: *const dyn DataLink) -> DynMetadata<dyn DataLink> {
    // SAFETY: Accessing the value from the `PtrRepr` union is safe since *const dyn DataLink
    // and PtrComponents have the same memory layouts. Only std can make this
    // guarantee.
    unsafe { PtrRepr { const_ptr: ptr }.components.metadata }
}

#[inline]
pub const fn from_raw_parts_mut(
    data_pointer: *mut (),
    metadata: DynMetadata<dyn DataLink>,
) -> *mut dyn DataLink {
    // SAFETY: Accessing the value from the `PtrRepr` union is safe since *const dyn DataLink
    // and PtrComponents have the same memory layouts. Only std can make this
    // guarantee.
    unsafe {
        PtrRepr {
            components: PtrComponents {
                data_pointer,
                metadata,
            },
        }
        .mut_ptr
    }
}

#[repr(C)]
union PtrRepr {
    const_ptr: *const dyn DataLink,
    mut_ptr: *mut dyn DataLink,
    components: PtrComponents,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct PtrComponents {
    data_pointer: *const (),
    metadata: DynMetadata<dyn DataLink>,
}

pub struct DynMetadata<Dyn: ?Sized> {
    vtable_ptr: &'static (),
    phantom: core::marker::PhantomData<Dyn>,
}

unsafe impl<Dyn: ?Sized> Send for DynMetadata<Dyn> {}
unsafe impl<Dyn: ?Sized> Sync for DynMetadata<Dyn> {}

impl<Dyn: ?Sized> fmt::Debug for DynMetadata<Dyn> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DynMetadata")
            .field(&(self.vtable_ptr as *const ()))
            .finish()
    }
}

// Manual impls needed to avoid `Dyn: $Trait` bounds.

impl<Dyn: ?Sized> Unpin for DynMetadata<Dyn> {}

impl<Dyn: ?Sized> Copy for DynMetadata<Dyn> {}

impl<Dyn: ?Sized> Clone for DynMetadata<Dyn> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
