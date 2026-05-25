//! Agent 八卦编排器
//!
//! 八个 Agent 对应八个卦象角色：
//!   总协调 / 知识库 / 代码生成 / 推理 /
//!   安全审计 / 并行搜索 / 网络调用 / 用户交互
//!
//! 阻抗矩阵动态决定 Agent 间信息流。

use crate::kernel::polarity::{ImpedanceMatrix, PolarityGenerator, PolarityVector, POLARITY_DIM};

/// Agent 角色
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentRole {
    Coordinator,    // 总协调
    KnowledgeBase,  // 知识库
    CodeGen,        // 代码生成（高因果强度）
    Reasoner,       // 推理/不确定性处理
    Auditor,        // 安全审计
    Searcher,       // 并行搜索/爬虫
    NetworkCaller,  // 网络/API 调用
    UserInterface,  // 用户交互/反馈
}

/// Agent 八卦编排器
pub struct AgentOrchestrator {
    /// 各 Agent 的极性向量
    polarities: [PolarityVector; 8],
    /// 阻抗矩阵
    impedance: ImpedanceMatrix,
}

impl AgentOrchestrator {
    pub fn new() -> Self {
        Self {
            polarities: [[0.0; POLARITY_DIM]; 8],
            impedance: ImpedanceMatrix::default(),
        }
    }

    /// 每轮重新计算 Agent 间的阻抗关系
    pub fn recompute(&mut self) {
        for i in 0..8 {
            for j in 0..8 {
                if i != j {
                    self.impedance.data[i][j] =
                        PolarityGenerator::compute_impedance(&self.polarities[i], &self.polarities[j]);
                }
            }
        }
        self.impedance.zero_diagonal();
    }

    /// 获取 Agent i 到 Agent j 的信息通行量
    /// transfer = 1 / (1 + impedance)
    pub fn transfer_rate(&self, from: usize, to: usize) -> f32 {
        1.0 / (1.0 + self.impedance.get(from, to))
    }
}
