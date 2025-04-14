use std::{
    ptr::NonNull,
    sync::{Mutex, OnceLock},
};

use allocator_api2::alloc::{AllocError, Allocator};
use bumpalo::Bump;

static BUMP: OnceLock<Mutex<Bump>> = OnceLock::new();

fn get_bump() -> &'static Mutex<Bump> {
    BUMP.get_or_init(|| Mutex::new(Bump::with_capacity(32 * 1024 * 1024))) // 32 MiB capacity
}

pub struct SignalAlloc;

unsafe impl Allocator for SignalAlloc {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        let bump = &*get_bump().lock().map_err(|_| AllocError)?;
        bump.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: std::alloc::Layout) {
        let Ok(bump) = get_bump().lock() else {
            // This will leak memory, but we can't do anything about it.
            return;
        };
        // SAFETY: We are using the same allocator, so this is safe.
        unsafe { (&*bump).deallocate(ptr, layout) };
    }
}
