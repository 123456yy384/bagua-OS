# bagua-OS
bagua OS (Used to prove that the scope of application of the gossip architecture is not limited to AI)
# BaGua OS

An **x86_64 operating system kernel built from scratch**, powered by the BaGua Architecture — a polarity-driven dynamic impedance scheduling system.

Instead of fixed timeslices, BaGua OS models hardware resource contention with an 8×8 impedance matrix and redistributes CPU/GPU/NPU quotas per tick based on polarity vectors. The scheduling algorithm is derived from Transformer attention mechanisms, ported to kernel space.

---

## Quick Start

```bash
# Run all tests (16 suites)
cargo test

# Userspace simulator (5000 tick scheduling loop, no QEMU needed)
cargo run --bin bagua-os

# Build bare-metal kernel (requires Rust nightly + x86_64-unknown-none target)
cd kernel && cargo +nightly build --release --target x86_64-unknown-none

# QEMU boot (requires QEMU 7.x + Limine)
make run
```

## Architecture

```
PIT 8253 (100Hz)
    │ timer_handler()
    ▼
BaGua Scheduler (polarity-driven dynamic impedance)
    │ 8×8 impedance matrix + per-tick quota recalculation
    ▼
┌──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┐
│ CPU  │ CBN  │ NPU  │ QPU  │ STO  │ NET  │ GPU  │ SIO  │
└──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┘
```

## Boot Flow

```
Multiboot2 → start_32 (32-bit trampoline)
  → CPUID check → Page Tables → PAE → EFER.LME → Paging
  → GDT switch → _start64 (64-bit)
  → Rust _start → Serial/GDT/IDT/PIT/Heap/Scheduler → Main loop
```

## Modules

| Module | Purpose |
|--------|---------|
| `kernel/src/boot64.S` | 32→64 long mode trampoline (GAS assembly) |
| `kernel/src/main.rs` | Kernel entry + Multiboot2/PVH headers |
| `kernel/src/gdt.rs` | Global Descriptor Table |
| `kernel/src/interrupts.rs` | IDT + interrupt handlers |
| `kernel/src/pit.rs` | 8253/8254 Programmable Interval Timer |
| `kernel/src/serial.rs` | COM1 serial output |
| `kernel/src/memory.rs` | Page tables + memory management |
| `kernel/src/allocator.rs` | Kernel heap allocator (64MB) |
| `kernel/src/elf.rs` | ELF loader |
| `kernel/src/syscall.rs` | System call interface |
| `src/kernel/scheduler.rs` | BaGua scheduler core |
| `src/hal/device.rs` | Hardware abstraction layer |

## Tech Stack

- **Language:** Rust (no_std, nightly)
- **Build:** Cargo + GNU as (via WSL)
- **Linker:** Custom linker.ld
- **Boot:** Multiboot2 + Limine
- **Emulation:** QEMU 7.x (x86_64-softmmu)

## Acceptance Status

| Item | Status |
|------|--------|
| cargo test | ✅ 16/16 passed |
| cargo run (simulator) | ✅ 5000 tick loop |
| cargo build (bare-metal) | ✅ 111KB ELF |
| QEMU bare-metal boot | ⚠️ Requires QEMU 7.2 + Limine |

See `TECH_ISSUES.md` for open technical challenges.
