pub mod scheduler;
pub mod polarity;
pub mod audit;
pub mod elimination;
pub mod buffer;
pub mod self_comp;
pub mod memory;
pub mod task_gate;
pub mod process;
#[cfg(not(feature = "no_std"))]
pub mod pagetable;
pub mod interrupt;
#[cfg(not(feature = "no_std"))]
pub mod fork_exec;
#[cfg(not(feature = "no_std"))]
pub mod vfs;

/// 八个卦象/子系统编号常量
///
/// 卦名不重要，本质是 8 个独立计算槽位（slot），
/// 通过极性向量和阻抗矩阵动态互连。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TrigramSlot {
    Slot0 = 0, // CPU 通用计算
    Slot1 = 1, // 碳基芯片调度层
    Slot2 = 2, // NPU 推理加速
    Slot3 = 3, // 量子计算单元
    Slot4 = 4, // 存储 / Memory
    Slot5 = 5, // 网络 / 互联
    Slot6 = 6, // GPU 并行计算
    Slot7 = 7, // 传感器 / IO
}

pub const NUM_SLOTS: usize = 8;

impl TrigramSlot {
    pub const ALL: [TrigramSlot; 8] = [
        TrigramSlot::Slot0,
        TrigramSlot::Slot1,
        TrigramSlot::Slot2,
        TrigramSlot::Slot3,
        TrigramSlot::Slot4,
        TrigramSlot::Slot5,
        TrigramSlot::Slot6,
        TrigramSlot::Slot7,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }
}
