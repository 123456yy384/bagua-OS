//! 任务自我感知 — 自适应内核模式
//!
//! 对应 AI 层：`RenWuZiWoGanZhi`
//!
//! OS 层简化为 5 种场景（原 AI 层 23 种对 OS 过度细分）：
//!   - 计算型 (compute)：编译、批量计算、代码生成 → 单向流水线
//!   - 查询型 (query)：数据库查询、检索、分类 → 双向交互
//!   - 流式 (stream)：媒体播放、网络服务 → 单向输出
//!   - 批处理 (batch)：标注、审核、预处理 → 平衡模式
//!   - 混合 (mixed)：对话、问答、指令执行 → 默认模式
//!
//! 从进程的前 N 个系统调用推断任务类型（非第一个 syscall，因为动态链接器干扰），
//! 输出 causal_strength，内核据此动态选择调度策略。

/// 任务类型（5 种，从 23 种精简）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskClass {
    /// 计算型：单向流水线，预取激进，大 buffer
    Compute,
    /// 查询型：双向交互，小 buffer，cache 偏 write-back
    Query,
    /// 流式：单向输出，预取激进，cache 偏 write-through
    Stream,
    /// 批处理：平衡模式
    Batch,
    /// 混合：默认模式
    Mixed,
}

impl TaskClass {
    /// 因果强度：0=双向/理解，1=单向/生成
    pub fn causal_strength(self) -> f32 {
        match self {
            TaskClass::Compute => 0.95,
            TaskClass::Query => 0.10,
            TaskClass::Stream => 0.90,
            TaskClass::Batch => 0.50,
            TaskClass::Mixed => 0.50,
        }
    }

    /// 从系统调用特征推断任务类型
    ///
    /// 启发式规则（可后续升级为在线学习）：
    ///   - mmap + brk 密集型 → Compute
    ///   - read/write 密集型且交错 → Query
    ///   - write 为主且连续 → Stream
    ///   - 无明显特征 → Batch 或 Mixed
    pub fn from_syscall_pattern(syscalls: &[u64]) -> Self {
        // TODO: 需要实际 syscall number 映射
        // 占位：默认返回 Mixed
        let _ = syscalls;
        TaskClass::Mixed
    }
}

/// 任务感知器
pub struct TaskPerceptor {
    /// 采样窗口大小（syscall 数量）
    #[allow(dead_code)]
    sample_window: usize,
    /// 当前识别的任务类型
    current_class: TaskClass,
}

impl TaskPerceptor {
    pub fn new() -> Self {
        Self {
            sample_window: 8,
            current_class: TaskClass::Mixed,
        }
    }

    /// 从系统调用序列识别任务类型
    pub fn identify(&mut self, syscalls: &[u64]) -> TaskClass {
        self.current_class = TaskClass::from_syscall_pattern(syscalls);
        self.current_class
    }

    /// 获取当前因果强度
    pub fn causal_strength(&self) -> f32 {
        self.current_class.causal_strength()
    }

    /// 选择调度策略
    pub fn select_strategy(&self) -> ScheduleStrategy {
        let cs = self.causal_strength();
        if cs > 0.7 {
            ScheduleStrategy::Unidirectional
        } else if cs < 0.3 {
            ScheduleStrategy::Bidirectional
        } else {
            ScheduleStrategy::Mixed
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ScheduleStrategy {
    /// 单向流水线：预取激进、buffer 大、cache 偏 write-through
    Unidirectional,
    /// 双向交互：预取消极、buffer 小、cache 偏 write-back
    Bidirectional,
    /// 混合模式
    Mixed,
}
