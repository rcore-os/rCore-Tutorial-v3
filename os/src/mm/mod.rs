mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use address::VPNRange;
pub use address::{PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
pub use frame_allocator::{FrameTracker, frame_alloc, frame_alloc_more, frame_dealloc};
pub use memory_set::{KERNEL_SPACE, MapArea, MapPermission, MapType, MemorySet, kernel_token};
use page_table::PTEFlags;
pub use page_table::{
    PageTable, PageTableEntry, UserBuffer, translated_byte_buffer, translated_ref,
    translated_refmut, translated_str,
};

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}
