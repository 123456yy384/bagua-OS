pub mod device;
#[cfg(not(feature = "no_std"))]
pub mod compat;

/// 硬件抽象层 — 卦象设备阵列
///
/// 八个卦象 slot 映射到物理硬件：
///   Slot0 — CPU      Slot4 — 存储/Memory
///   Slot1 — 碳基调度  Slot5 — 网络/互联
///   Slot2 — NPU      Slot6 — GPU
///   Slot3 — 量子计算  Slot7 — 传感器/IO
pub use device::DeviceClass;
