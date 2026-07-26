use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            let allocated = ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            let live = allocated.saturating_sub(DEALLOCATED.load(Ordering::Relaxed));
            PEAK.fetch_max(live, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        DEALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

#[unsafe(no_mangle)]
pub extern "C" fn jb_rust_heap_used_bytes() -> usize {
    ALLOCATED
        .load(Ordering::Relaxed)
        .saturating_sub(DEALLOCATED.load(Ordering::Relaxed))
}

#[unsafe(no_mangle)]
pub extern "C" fn jb_rust_heap_peak_bytes() -> usize {
    PEAK.load(Ordering::Relaxed)
}
