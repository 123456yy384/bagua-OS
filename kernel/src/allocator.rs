//! 简易 bump allocator — 内核堆分配
//!
//! 为裸机提供 `Box`, `Vec` 等容器所需的内存分配。
//! 使用固定大小堆区域，线性分配，不支持释放。

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

/// 堆大小：64 MB
const HEAP_SIZE: usize = 64 * 1024 * 1024;
/// 堆起始地址（高地址区域）
const HEAP_START: usize = 0xFFFF_8000_0000_0000;

/// 全局分配器实例
#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new();

pub struct BumpAllocator {
    next: AtomicUsize,
    end: usize,
}

impl BumpAllocator {
    const fn new() -> Self {
        Self {
            next: AtomicUsize::new(HEAP_START),
            end: HEAP_START + HEAP_SIZE,
        }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();

        let next = self.next.load(Ordering::Relaxed);
        let aligned = (next + align - 1) & !(align - 1);

        if aligned + size > self.end {
            return core::ptr::null_mut();
        }

        self.next.store(aligned + size, Ordering::Relaxed);
        aligned as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // bump allocator 不支持释放
    }
}

/// 初始化堆（标记堆区域可用）
pub fn init_heap() {
    // bump allocator 自带静态初始化，无需额外操作
}
