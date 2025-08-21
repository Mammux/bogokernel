use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOC: LockedHeap = LockedHeap::empty();

extern "C" {
    static __heap_start: u8;
    static __heap_end: u8;
}

pub fn init() {
    let start = unsafe { &__heap_start as *const u8 as usize };
    let end   = unsafe { &__heap_end   as *const u8 as usize };
    let size  = end - start;
    assert!(size > 0, "heap size must be > 0");
    unsafe { ALLOC.lock().init(start as *mut u8, size) };
}
