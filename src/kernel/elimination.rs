//! 低效淘汰 — 动态资源回收
//!
//! 对应 AI 层：`TaoTaiDiXiaoJiZhi`
//!
//! 连续 N 个审计周期质量低于存活阈值 → 进程冻结 + 归档
//! 类似 OOM killer 但更精细：不是杀，是冷冻 + 可恢复。
//!
//! TODO: 冻结 vs 杀掉 — 归档到 swap 的具体策略
//! TODO: 恢复机制：什么条件触发解冻？
//! TODO: 高存活阈值模式下（logic_pressure > 1.5）的行为差异

/// 淘汰器
pub struct Eliminator {
    /// 存活阈值
    pub survival_threshold: f32,
    /// 连续不合格次数上限
    pub max_strikes: u8,
}

impl Eliminator {
    pub fn new(survival_threshold: f32) -> Self {
        Self {
            survival_threshold,
            max_strikes: 3,
        }
    }

    /// 判断一个进程是否应被淘汰
    pub fn should_eliminate(&self, quality: f32, consecutive_strikes: u8) -> EliminationAction {
        if quality >= self.survival_threshold {
            EliminationAction::Keep
        } else if consecutive_strikes >= self.max_strikes {
            EliminationAction::Freeze
        } else {
            EliminationAction::Warn
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum EliminationAction {
    Keep,   // 存活，继续运行
    Warn,   // 警告，给一���自检机会
    Freeze, // 冻结 + 归档到 swap
}
