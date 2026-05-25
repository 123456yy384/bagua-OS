//! 算力缓冲区 — 内核状态平滑层
//!
//! 对应 AI 层：`SuanLiHuanChongQu`
//!
//! OS 层裁切：使用成熟 CPU governor 的 PID 平滑策略，
//! 而非 AI 层的"可学习门控"。
//!
//! 策略：
//!   - 与 Linux cpufreq 的 schedutil governor 兼容
//!   - 采用指数加权移动平均 (EWMA) 平滑负载跳变
//!   - 压力模式下提高响应速度（缩短平滑窗口）

/// 内核状态平滑器（EWMA + 自适应窗口）
pub struct LoadSmoother {
    /// EWMA 衰减因子（0-1，越大越平滑）
    alpha: f32,
    /// 上一周期的各 slot 负载
    prev_loads: [f32; 8],
    /// 当前 logic_pressure（影响平滑速度）
    logic_pressure: f32,
}

impl LoadSmoother {
    /// 默认衰减因子 0.3（约 3 周期过渡）
    pub fn new() -> Self {
        Self {
            alpha: 0.3,
            prev_loads: [0.0; 8],
            logic_pressure: 1.0,
        }
    }

    /// 设置全局压力（调节平滑速度）
    pub fn set_pressure(&mut self, pressure: f32) {
        self.logic_pressure = pressure;
    }

    /// EWMA 平滑当前负载
    ///
    /// smoothed = alpha × prev + (1 - alpha) × current
    ///
    /// 高压模式下 alpha 减小（更快响应变化）
    pub fn smooth(&mut self, current_loads: &[f32; 8]) -> [f32; 8] {
        // 压力越大，窗口越小（更快响应）
        let effective_alpha = self.alpha / self.logic_pressure;

        let mut smoothed = [0.0f32; 8];
        for i in 0..8 {
            smoothed[i] = effective_alpha * self.prev_loads[i]
                + (1.0 - effective_alpha) * current_loads[i];
        }

        self.prev_loads = smoothed;
        smoothed
    }
}
