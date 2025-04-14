use std::{
    alloc::Layout,
    ptr::NonNull,
    sync::{LazyLock, Mutex},
};

use allocator_api2::alloc::{AllocError, Allocator};
use bumpalo::Bump;

const INITIAL_CAPACITY: usize = 64 * 1024 * 1024; // 64 MiB

static BUMP: LazyLock<Mutex<Bump>> =
    LazyLock::new(|| Mutex::new(Bump::with_capacity(INITIAL_CAPACITY)));

// pub type SignalAlloc = std::alloc::System;

#[derive(Default, Clone, Copy)]
pub struct SignalAlloc {}

unsafe impl Allocator for SignalAlloc {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let bump = &*BUMP.lock().map_err(|_| AllocError)?;
        bump.allocate(layout)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let Ok(bump) = BUMP.lock() else {
            // This will leak memory, but we can't do anything about it.
            return;
        };
        // SAFETY: We are using the same allocator, so this is safe.
        unsafe { (&*bump).deallocate(ptr, layout) };
    }
}
