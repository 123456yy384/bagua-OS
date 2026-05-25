//! QEMU bare-metal boot target
//!
//! 编译为 x86_64 裸机内核镜像。
//! 需要：
//!   - bootloader (limine / multiboot2)
//!   - 链接脚本
//!   - 裸机启动入口
//!
//! 构建命令：
//!   cargo build --target x86_64-unknown-none --features no_std
//!
//! 当前为骨架，需要后续填充。

#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// 裸机 panic 处理
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// 内核入口（由 bootloader 调用）
///
/// TODO: 实际启动流程：
///   1. 初始化 GDT/IDT
///   2. 初始化页表
///   3. 初始化八卦调度器
///   4. 进入调度循环
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // TODO: 早期初始化

    loop {
        // 主循环：中断驱动的八卦调度
        // 当前占位
    }
}

/// 链接脚本指定这些符号
extern "C" {
    static __kernel_start: u8;
    static __kernel_end: u8;
}
