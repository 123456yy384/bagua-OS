//! Linux 驱动兼容层 — 内核符号映射
//!
//! 目标：直接加载 Linux 内核模块（.ko），
//! 将 Linux 内核 API 映射到八卦九模块实现。
//!
//! 兼容符号表：
//!   kmalloc/vmalloc → 九州地址分配器
//!   kfree/vfree     → 九州地址释放
//!   request_irq     → 八卦中断控制器
//!   printk          → 八卦日志子系统
//!
//! TODO: 完整的 .ko ELF 加载器

use crate::kernel::memory::JiuzhouAddress;

/// kmalloc 分配（返回九州地址）
pub fn kmalloc(size: usize, _flags: u32) -> Option<*mut u8> {
    // TODO: 九州地址分配器
    // 当前返回堆分配指针（模拟模式）
    if size == 0 {
        return None;
    }
    let layout = std::alloc::Layout::from_size_align(size, 16).ok()?;
    let ptr = unsafe { std::alloc::alloc(layout) };
    if ptr.is_null() { None } else { Some(ptr) }
}

/// kfree 释放
pub fn kfree(ptr: *mut u8, size: usize) {
    if !ptr.is_null() && size > 0 {
        if let Ok(layout) = std::alloc::Layout::from_size_align(size, 16) {
            unsafe { std::alloc::dealloc(ptr, layout) }
        }
    }
}

/// 物理地址 → 九州地址
pub fn phys_to_jiuzhou(phys: u64) -> JiuzhouAddress {
    JiuzhouAddress::from_physical(phys)
}

/// 九州地址 → 物理地址
pub fn jiuzhou_to_phys(addr: &JiuzhouAddress) -> u64 {
    addr.to_physical()
}

/// 打印内核日志（映射到 println!）
pub fn printk(fmt: &str) {
    println!("[bagua-os] {}", fmt);
}

/// 注册中断处理
pub fn request_irq(_irq: u8, _handler: fn()) -> Result<(), CompatError> {
    // TODO: 接入八卦中断控制器
    Ok(())
}

#[derive(Debug)]
pub enum CompatError {
    NotImplemented,
    ModuleLoadFailed,
    SymbolNotFound,
    OutOfMemory,
}
