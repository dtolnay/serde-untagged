use std::any::{self, TypeId};
use std::marker::PhantomData;
use std::mem;

pub(crate) struct ErasedValue {
    ptr: *mut (),
    drop: unsafe fn(*mut ()),
    type_id: TypeId,
    type_name: &'static str,
}

impl ErasedValue {
    pub(crate) unsafe fn new<T>(value: T) -> Self {
        ErasedValue {
            ptr: Box::into_raw(Box::new(value)).cast(),
            drop: {
                unsafe fn drop<T>(ptr: *mut ()) {
                    let _ = Box::from_raw(ptr.cast::<T>());
                }
                drop::<T>
            },
            type_id: non_static_type_id::<T>(),
            type_name: any::type_name::<T>(),
        }
    }

    pub(crate) unsafe fn take<T>(self) -> T {
        if cfg!(debug_assertions) && self.type_id != non_static_type_id::<T>() {
            panic!(
                "ErasedValue mismatch: {} vs {}",
                self.type_name,
                any::type_name::<T>(),
            );
        }

        let b = Box::from_raw(self.ptr.cast::<T>());
        mem::forget(self);
        *b
    }
}

impl Drop for ErasedValue {
    fn drop(&mut self) {
        unsafe { (self.drop)(self.ptr) }
    }
}

trait NonStaticAny {
    fn get_type_id(&self) -> TypeId
    where
        Self: 'static;
}

impl<T: ?Sized> NonStaticAny for PhantomData<T> {
    fn get_type_id(&self) -> TypeId
    where
        Self: 'static,
    {
        TypeId::of::<T>()
    }
}

fn non_static_type_id<T: ?Sized>() -> TypeId {
    let phantom_data = PhantomData::<T>;
    NonStaticAny::get_type_id(unsafe {
        mem::transmute::<&dyn NonStaticAny, &(dyn NonStaticAny + 'static)>(&phantom_data)
    })
}
