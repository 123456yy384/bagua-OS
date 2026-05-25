pub mod orchestrator;

/// Agent 八卦编排层
///
/// 八个 Agent 各自承担一个卦象角色，
/// 通过阻抗矩阵动态决定信息流。
pub use orchestrator::AgentRole;
