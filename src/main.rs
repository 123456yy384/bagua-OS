//! 八卦 OS 入口
//!
//! 当前为用户态模拟 binary target，
//! 后续替换为裸机 bootloader 入口。
//!
//! 模拟运行：cargo run --bin bagua-os

use bagua_os::hal::device::{DeviceClass, DeviceState};
use bagua_os::kernel::scheduler::BaguaScheduler;

fn main() {
    println!("=== 八卦 OS v0.1.0 ===");
    println!("极性驱动动态阻抗调度系统\n");

    // 创建 8 个设备 slot
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
            load: 0.3 + (i as f32) * 0.08,
            temperature: 40.0 + (i as f32) * 5.0,
            power_draw: 50.0 + (i as f32) * 25.0,
        }
    });

    let mut sched = BaguaScheduler::new(slots);

    // 生成进程（给初始工作量避免审计秒杀）
    for slot in 0..8 {
        for _ in 0..3 {
            if let Some(pid) = sched.spawn(slot as u8) {
                // 给新进程一些初始统计值
                if let Some(proc) = sched.proc_table.find_mut(pid) {
                    proc.cpu_ticks = 500 + (slot as u64) * 100;
                    proc.io_bytes = 2048 + (slot as u64) * 512;
                }
            }
        }
    }
    println!("生成了 {} 个进程", sched.proc_table.active_count());

    // 模拟运行
    let total_ticks = 5000;
    println!("运行 {} 个调度周期...\n", total_ticks);

    for t in 0..total_ticks {
        sched.tick();

        // 模拟进程活动：每 10 tick 给随机进程增加统计
        if t % 10 == 0 {
            for proc in sched.proc_table.active_procs_mut() {
                proc.cpu_ticks = proc.cpu_ticks.wrapping_add(10);
                proc.io_bytes = proc.io_bytes.wrapping_add(4);
            }
        }

        if t % 500 == 0 {
            println!(
                "tick {:>6} | pressure={:.3} | active={:>3} | frozen={:>3}",
                t,
                sched.health.logic_pressure,
                sched.proc_table.active_count(),
                sched.proc_table.active_procs()
                    .filter(|p| matches!(p.state, bagua_os::kernel::process::ProcState::Frozen))
                    .count(),
            );
        }
    }

    // 最终状态
    println!("\n--- 最终状态 ---");
    println!("总 tick:    {}", sched.tick);
    println!("逻辑压力:   {:.3}", sched.health.logic_pressure);
    println!("活跃进程:   {}", sched.proc_table.active_count());

    println!("\nslot 配额分配:");
    for i in 0..8 {
        let cls = match i {
            0 => "CPU",
            1 => "CBN",
            2 => "NPU",
            3 => "QPU",
            4 => "STO",
            5 => "NET",
            6 => "GPU",
            _ => "SIO",
        };
        println!(
            "  [{i}] {cls:>3} | quota={:>4} | score={:.3}",
            sched.slot_quota[i],
            sched.slot_scores[i],
        );
    }

    println!("\n阻抗矩阵 (8×8):");
    for i in 0..8 {
        for j in 0..8 {
            if i != j {
                print!(" {:>5.1}", sched.impedance.get(i, j));
            } else {
                print!("    ·");
            }
        }
        println!();
    }

    println!("\n八卦 OS 模拟完成。");
}
