//! 去中心化自计算 — 分布式内核健康监控
//!
//! 对应 AI 层：`logic_pressure` 自动调节
//!
//! 公式：
//!   avg_survival = sum(scores) / 8
//!   logic_pressure = clamp(pressure + 0.05 × (1 - avg_survival), 0.5, 2.0)
//!
//! 内核模式：
//!   pressure ≈ 0.5 → 宽松模式（大量任务，低淘汰频率）
//!   pressure ≈ 1.0 → 标准模式
//!   pressure ≈ 2.0 → 高压模式（严格淘汰，短 quantum）
//!
//! TODO: 八个子系统的存活率如何各自独立计算
//! TODO: 级联故障防护 — 一个子系统崩溃不等于全局恐慌
//! TODO: logic_pressure 的上升/下降速率调优

/// 内核健康监控器（去中心化）
pub struct HealthMonitor {
    /// 全局逻辑压力（0.5-2.0）
    pub logic_pressure: f32,
    /// 每个 slot 的存活分数历史
    slot_survival_history: [f32; 8],
    /// 调整步长
    adjustment_step: f32,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            logic_pressure: 1.0,
            slot_survival_history: [1.0; 8],
            adjustment_step: 0.05,
        }
    }

    /// 输入本轮各 slot 的存活分数，输出调整后的 logic_pressure
    pub fn update(&mut self, slot_scores: &[f32; 8]) -> f32 {
        let avg_survival: f32 = slot_scores.iter().sum::<f32>() / 8.0;
        self.slot_survival_history = *slot_scores;

        // 存活率下降 → 压力上升
        // 存活率上升 → 压力下降
        self.logic_pressure += self.adjustment_step * (1.0 - avg_survival);
        self.logic_pressure = self.logic_pressure.clamp(0.5, 2.0);

        self.logic_pressure
    }

    /// 当前压力水平对应的工作模式
    pub fn current_mode(&self) -> KernelMode {
        if self.logic_pressure < 0.7 {
            KernelMode::Relaxed
        } else if self.logic_pressure < 1.3 {
            KernelMode::Standard
        } else {
            KernelMode::HighPressure
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum KernelMode {
    /// 宽松模式：量子周期长、淘汰阈值低、容许更多低效进程
    Relaxed,
    /// 标凅模式
    Standard,
    /// 高压模式：量子周期短、淘汰阈值高、快速回收资源
    HighPressure,
}
