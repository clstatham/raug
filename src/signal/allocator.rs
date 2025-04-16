use allocator_api2::alloc::Global;
use blink_alloc::SyncBlinkAlloc;

const INITIAL_CAPACITY: usize = 64 * 1024 * 1024; // 64 MiB

pub static SIGNAL_ALLOC: SyncBlinkAlloc =
    SyncBlinkAlloc::with_chunk_size_in(INITIAL_CAPACITY, Global);
