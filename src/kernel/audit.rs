//! 淘汰审核 — 进程质量审计
//!
//! 对应 AI 层：`TaoTaiShenHe`
//!
//! 每个进程维护质量分数 = 资源效率 × 因果有效性：
//!   - 资源效率：有效 CPU 时间 / 总 CPU 时间
//!   - 因果有效性：输出量 / 资源消耗量
//!
//! 低于阈值不直接 kill，发送 SIG_AUDIT 信号给进程自检机会。
//!
//! TODO: 如何定义"输出量"——进程的 I/O 量？系统调用频率？页面写回次数？
//! TODO: SIG_AUDIT 的用户态处理协议
//! TODO: honesty_loss 的 OS 层等价物是什么？

/// 进程质量审计器
pub struct ProcessAuditor {
    /// 每轮审计的时间片（tick 数）
    pub audit_interval: u64,
    /// 质量分数阈值（低于此值触发审计信号）
    pub quality_threshold: f32,
}

/// 单个进程的质量评分
#[derive(Clone)]
pub struct QualityScore {
    /// 资源效率（0-1）
    pub resource_efficiency: f32,
    /// 因果有效性（输入输出差值归一化）
    pub causal_validity: f32,
    /// 综合质量 = resource_efficiency × causal_validity
    pub composite: f32,
}

impl ProcessAuditor {
    pub fn new(audit_interval: u64, quality_threshold: f32) -> Self {
        Self {
            audit_interval,
            quality_threshold,
        }
    }

    /// 审计一个进程
    ///
    /// 评分公式：
    ///   efficiency = clamp(cpu_time / (tick_window × active_proc_count), 0, 1)
    ///   validity   = clamp(io_bytes / 4096, 0, 1)
    ///   composite  = efficiency × validity
    ///
    /// 新进程（无统计）默认给中位分 0.5。
    pub fn evaluate(&self, _proc_id: u64, cpu_time: u64, io_bytes: u64, _mem_pages: u64) -> QualityScore {
        // 新进程默认中位分
        if cpu_time == 0 && io_bytes == 0 {
            return QualityScore {
                resource_efficiency: 0.5,
                causal_validity: 0.5,
                composite: 0.25,
            };
        }

        let efficiency = (cpu_time as f32 / 1000.0).clamp(0.0, 1.0);
        let validity = (io_bytes as f32 / 4096.0).clamp(0.0, 1.0);
        QualityScore {
            resource_efficiency: efficiency,
            causal_validity: validity,
            composite: efficiency * validity,
        }
    }

    /// 决定是否发送审计信号
    pub fn should_audit(&self, score: &QualityScore) -> bool {
        score.composite < self.quality_threshold
    }
}
