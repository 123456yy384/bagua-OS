//! 中断控制器 + 时钟模拟
//!
//! 当前为模拟模式（用户态 timer），
//! 裸机模式替换为 APIC / RISC-V PLIC。

/// 中断向量
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Irq {
    Timer = 0,
    Keyboard = 1,
    // 可扩展...
}

/// 中断帧（保存的寄存器状态）
#[derive(Debug, Clone, Copy)]
pub struct InterruptFrame {
    pub irq: Irq,
    pub error_code: u64,
}

/// 中断控制器
pub struct InterruptController {
    /// 是否启用中断
    enabled: bool,
    /// 挂起的中断队列
    pending: heapless::Vec<Irq, 32>,
    /// 时钟 tick 计数器
    timer_ticks: u64,
    /// 时钟频率（Hz）
    #[allow(dead_code)]
    timer_freq_hz: u64,
}

impl InterruptController {
    pub fn new() -> Self {
        Self {
            enabled: false,
            pending: heapless::Vec::new(),
            timer_ticks: 0,
            timer_freq_hz: 100, // 100Hz = 10ms per tick
        }
    }

    /// 启用中断
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// 禁用中断
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// 时钟推进（模拟硬件时钟）
    pub fn tick(&mut self) {
        self.timer_ticks += 1;
        if self.enabled {
            let _ = self.pending.push(Irq::Timer);
        }
    }

    /// 触发外部中断
    pub fn trigger(&mut self, irq: Irq) {
        if self.enabled {
            let _ = self.pending.push(irq);
        }
    }

    /// 获取下一个待处理中断（无中断返回 None）
    pub fn next_irq(&mut self) -> Option<Irq> {
        if !self.enabled || self.pending.is_empty() {
            None
        } else {
            Some(self.pending.remove(0))
        }
    }

    /// 当前时钟 tick
    pub fn elapsed_ticks(&self) -> u64 {
        self.timer_ticks
    }
}


