use std::mem;

pub(crate) struct ErasedValue {
    ptr: *mut (),
    drop: unsafe fn(*mut ()),
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
        }
    }

    pub(crate) unsafe fn take<T>(self) -> T {
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
