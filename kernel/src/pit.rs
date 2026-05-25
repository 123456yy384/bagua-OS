//! PIT — 可编程间隔定时器
//!
//! 初始化 PIT 产生 100Hz 时钟中断 (IRQ0)。
//! 每个 tick 触发 timer_handler → on_timer_tick → 调度器。

use x86_64::instructions::port::PortWriteOnly;

/// PIT 通道 0 数据端口
const PIT_CHANNEL0: u16 = 0x40;
/// PIT 命令端口
const PIT_COMMAND: u16 = 0x43;

/// 初始化 PIT，设置频率为 100Hz
///
/// PIT 基础频率 = 1,193,182 Hz
/// divisor = 1,193,182 / 100 ≈ 11,932
pub fn init() {
    let divisor: u16 = 11932;

    unsafe {
        // 命令字：通道 0 | 低字节+高字节 | 模式 3 (方波) | 二进制
        let mut cmd = PortWriteOnly::new(PIT_COMMAND);
        cmd.write(0x36u8);

        // 写入分频值（低字节 → 高字节）
        let mut data = PortWriteOnly::new(PIT_CHANNEL0);
        data.write((divisor & 0xFF) as u8);
        data.write((divisor >> 8) as u8);
    }
}

/// 重新映射 PIC (可编程中断控制器)
///
/// 将 IRQ 0-7 映射到中断向量 32-39，
/// 避免与 CPU 异常向量 (0-31) 冲突。
pub fn remap_pic() {
    unsafe {
        let mut pic1_cmd = PortWriteOnly::new(0x20);
        let mut pic2_cmd = PortWriteOnly::new(0xA0);
        let mut pic1_data = PortWriteOnly::new(0x21);
        let mut pic2_data = PortWriteOnly::new(0xA1);

        // 保存掩码
        // 初始化序列 (ICW1)
        pic1_cmd.write(0x11u8);
        pic2_cmd.write(0x11u8);

        // ICW2: 基址向量
        pic1_data.write(32u8); // IRQ 0-7 → 向量 32-39
        pic2_data.write(40u8); // IRQ 8-15 → 向量 40-47

        // ICW3: 级联
        pic1_data.write(0x04u8); // PIC2 连接在 IRQ2
        pic2_data.write(0x02u8); // 级联标识

        // ICW4: x86 模式
        pic1_data.write(0x01u8);
        pic2_data.write(0x01u8);

        // 启用所有 IRQ
        pic1_data.write(0x00u8);
        pic2_data.write(0x00u8);
    }
}
