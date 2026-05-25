//! 八卦 OS 内核 — x86_64 裸机入口
//!
//! 启动流程：
//!   Multiboot2 → _start → GDT/IDT/PIC/PIT → 串口 → 调度器 → 中断循环

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod allocator;
mod gdt;
mod interrupts;
mod memory;
mod elf;
mod syscall;
mod pit;
mod serial;

use core::panic::PanicInfo;
use spin::Mutex;
use bagua_os::kernel::scheduler::BaguaScheduler;
use bagua_os::hal::device::{DeviceClass, DeviceState};

/// QEMU PVH ELF note — required for -kernel boot on QEMU 7.2+
#[unsafe(no_mangle)]
#[used]
#[link_section = ".note.pvh"]
static PVH_NOTE: [u8; 24] = {
    let mut n = [0u8; 24];
    n[0] = 4; n[1] = 0; n[2] = 0; n[3] = 0;
    n[4] = 4; n[5] = 0; n[6] = 0; n[7] = 0;
    n[8] = 18; n[9] = 0; n[10] = 0; n[11] = 0;
    n[12] = b'X'; n[13] = b'e'; n[14] = b'n'; n[15] = 0;
    n
};

/// Multiboot2 引导头（供 Limine/GRUB 使用）
#[unsafe(no_mangle)]
#[used]
#[link_section = ".multiboot2_header"]
static MULTIBOOT2_HEADER: [u8; 24] = {
    let mut h = [0u8; 24];
    h[0] = 0xD6; h[1] = 0x50; h[2] = 0x52; h[3] = 0xE8;
    h[8] = 24;
    let cs = (0xE852_50D6u32).wrapping_add(0).wrapping_add(24);
    let cs = 0u32.wrapping_sub(cs);
    h[12] = cs as u8; h[13] = (cs >> 8) as u8;
    h[14] = (cs >> 16) as u8; h[15] = (cs >> 24) as u8;
    h
};

/// 全局调度器
static SCHEDULER: Mutex<Option<BaguaScheduler>> = Mutex::new(None);

/// 上次打印 tick 号（避免串口刷屏）
static LAST_PRINT_TICK: Mutex<u64> = Mutex::new(0);

// QEMU debug port (0xE9) — no driver needed, works on any QEMU version
fn debug_out(byte: u8) {
    unsafe { core::arch::asm!("out dx, al", in("dx") 0xE9u16, in("al") byte) };
}

fn debug_str(s: &str) {
    for &b in s.as_bytes() { debug_out(b); }
    debug_out(b'\n');
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // First thing: write to QEMU debug port 0xE9
    // This uses only raw x86 OUT instruction — zero dependencies
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") 0xE9u16,
            in("al") b'B',
        );
        core::arch::asm!(
            "out dx, al",
            in("dx") 0xE9u16,
            in("al") b'!',
        );
    }

    // Now try full init
    debug_str("=== BaGua OS v0.1.0 ===");

    serial::init();
    debug_str("serial OK");

    gdt::init();
    interrupts::init_idt();
    pit::remap_pic();
    pit::init();
    allocator::init_heap();
    init_scheduler();
    debug_str("scheduler OK");

    x86_64::instructions::interrupts::enable();

    loop {
        x86_64::instructions::hlt();
    }
}

fn init_scheduler() {
    let slots: [DeviceState; 8] = core::array::from_fn(|i| {
        let class = match i {
            0 => DeviceClass::Cpu,
            1 => DeviceClass::CarbonScheduler,
            2 => DeviceClass::Npu,
            3 => DeviceClass::QuantumUnit,
            4 => DeviceClass::Storage,
            5 => DeviceClass::Network,
            6 => DeviceClass::Gpu,
            _ => DeviceClass::SensorIO,
        };
        DeviceState {
            class,
            load: 0.3,
            temperature: 40.0 + (i as f32) * 5.0,
            power_draw: 50.0 + (i as f32) * 25.0,
        }
    });

    let mut sched = BaguaScheduler::new(slots);
    for slot in 0..8 {
        for _ in 0..2 {
            if let Some(pid) = sched.spawn(slot) {
                if let Some(proc) = sched.proc_table.find_mut(pid) {
                    proc.cpu_ticks = 100;
                    proc.io_bytes = 100;
                }
            }
        }
    }
    *SCHEDULER.lock() = Some(sched);
}

/// 每个时钟 tick 调用
pub fn on_timer_tick() {
    if let Some(ref mut sched) = *SCHEDULER.lock() {
        sched.tick();

        // 每 500 tick 输出一次调度状态
        let mut last = LAST_PRINT_TICK.lock();
        if sched.tick - *last >= 500 {
            *last = sched.tick;

            serial_println!(
                "[t={:>6}] pressure={:.3}  active={:>3}  frozen={:>2}",
                sched.tick,
                sched.health.logic_pressure,
                sched.proc_table.active_count(),
                sched.proc_table.active_procs()
                    .filter(|p| matches!(p.state, bagua_os::kernel::process::ProcState::Frozen))
                    .count(),
            );

            // 配额 TOP 3（手动找最大）
            let names = ["CPU", "CBN", "NPU", "QPU", "STO", "NET", "GPU", "SIO"];
            let mut top: [(usize, u64); 3] = [(0, 0); 3];
            for i in 0..8 {
                let q = sched.slot_quota[i];
                if q > top[0].1 { top[2] = top[1]; top[1] = top[0]; top[0] = (i, q); }
                else if q > top[1].1 { top[2] = top[1]; top[1] = (i, q); }
                else if q > top[2].1 { top[2] = (i, q); }
            }
            serial_println!(
                "        quota: {}={} {}={} {}={}",
                names[top[0].0], top[0].1,
                names[top[1].0], top[1].1,
                names[top[2].0], top[2].1,
            );
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("KERNEL PANIC: {}", info);
    loop { x86_64::instructions::hlt(); }
}
