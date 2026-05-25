//! 卦象设备阵列 — 异构硬件抽象
//!
//! 每个 slot 对应一类物理硬件。
//! 核心机制：极性向量编码硬件特征，阻抗矩阵决定跨设备数据流。

use crate::kernel::TrigramSlot;

/// 设备类别
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceClass {
    Cpu,
    CarbonScheduler, // 碳基芯片调度层
    Npu,
    QuantumUnit,
    Storage,
    Network,
    Gpu,
    SensorIO,
}

impl DeviceClass {
    pub fn from_slot(slot: TrigramSlot) -> Self {
        match slot {
            TrigramSlot::Slot0 => DeviceClass::Cpu,
            TrigramSlot::Slot1 => DeviceClass::CarbonScheduler,
            TrigramSlot::Slot2 => DeviceClass::Npu,
            TrigramSlot::Slot3 => DeviceClass::QuantumUnit,
            TrigramSlot::Slot4 => DeviceClass::Storage,
            TrigramSlot::Slot5 => DeviceClass::Network,
            TrigramSlot::Slot6 => DeviceClass::Gpu,
            TrigramSlot::Slot7 => DeviceClass::SensorIO,
        }
    }

    /// 硬件能力特征（编码为极性向量的前几维）
    pub fn capability_vector(&self) -> [f32; 8] {
        match self {
            DeviceClass::Cpu             => [1.0, 0.0, 0.2, 0.0, 0.5, 0.0, 0.0, 0.0],
            DeviceClass::CarbonScheduler => [0.0, 1.0, 0.0, 0.0, 0.0, 0.9, 0.0, 0.0], // 超低功耗
            DeviceClass::Npu             => [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.8, 0.0], // 矩阵密集
            DeviceClass::QuantumUnit     => [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.5], // 概率振幅
            DeviceClass::Storage         => [0.0, 0.3, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
            DeviceClass::Network         => [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            DeviceClass::Gpu             => [0.0, 0.0, 0.4, 0.0, 0.0, 0.0, 1.0, 0.0],
            DeviceClass::SensorIO        => [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
        }
    }
}

/// 设备状态（用于极性生成）
pub struct DeviceState {
    pub class: DeviceClass,
    pub load: f32,
    pub temperature: f32,
    pub power_draw: f32,
}
