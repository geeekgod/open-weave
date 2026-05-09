// Mock allocator for no_std environments
#[cfg(feature = "edge")]
use alloc::alloc::{GlobalAlloc, Layout};

#[cfg(feature = "edge")]
struct BumpAllocator;

#[cfg(feature = "edge")]
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        std::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(feature = "edge")]
#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;