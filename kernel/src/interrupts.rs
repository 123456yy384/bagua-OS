//! IDT — 中断描述符表
//!
//! 配置 x86_64 中断处理：时钟、键盘、双重错误等。

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use spin::Lazy;

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    // 双重错误
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
    }

    // 页错误
    idt.page_fault.set_handler_fn(page_fault_handler);

    // 时钟中断（IRQ0 → 向量 32）
    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_handler);

    // 键盘中断（IRQ1 → 向量 33）
    idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_handler);

    idt
});

/// 初始化 IDT
pub fn init_idt() {
    IDT.load();
}

// ──── 中断处理 ────

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("DOUBLE FAULT at {:#x}", stack_frame.instruction_pointer.as_u64());
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    use x86_64::VirtAddr;
    panic!(
        "PAGE FAULT at {:#x}, address: {:#x}, error: {:?}",
        stack_frame.instruction_pointer.as_u64(),
        Cr2::read().unwrap_or(VirtAddr::zero()).as_u64(),
        error_code,
    );
}

/// 时钟 tick：每个 PIT 周期触发，驱动调度器
extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    crate::on_timer_tick();

    unsafe {
        // EOI (End of Interrupt) 信号
        x86_64::instructions::port::PortWriteOnly::new(0x20).write(0x20u8);
    }
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    // TODO: 键盘输入处理

    unsafe {
        x86_64::instructions::port::PortWriteOnly::new(0x20).write(0x20u8);
    }
}

// ──── 工具 ────

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    Timer = 32,
    Keyboard = 33,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        self as usize
    }
}
