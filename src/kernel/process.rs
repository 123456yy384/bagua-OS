//! 进程管理 — PCB + 进程表
//!
//! 固定大小进程表（no_std 下无动态分配），
//! 每个进程维护质量审计所需的统计数据。

use crate::kernel::audit::QualityScore;
use crate::kernel::task_gate::TaskClass;

/// 最大进程数
pub const MAX_PROCS: usize = 256;

/// 进程状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcState {
    Running,
    Ready,
    Blocked,
    Frozen,
    Zombie,
    Free,
}

/// 进程控制块
#[derive(Clone)]
pub struct Process {
    pub pid: u16,
    pub state: ProcState,
    /// 归属的 slot
    pub slot_id: u8,
    /// 任务类型
    pub task_class: TaskClass,
    /// 连续审计不合格次数
    pub audit_strikes: u8,
    // 统计信息
    pub cpu_ticks: u64,
    pub io_bytes: u64,
    pub page_faults: u64,
    pub syscall_count: u64,
    /// 最近 8 个系统调用号（用于任务识别）
    pub recent_syscalls: [u64; 8],
    pub syscall_idx: u8,
    /// 质量评分
    pub quality: QualityScore,
}

impl Process {
    pub fn new(pid: u16) -> Self {
        Self {
            pid,
            state: ProcState::Ready,
            slot_id: 0,
            task_class: TaskClass::Mixed,
            audit_strikes: 0,
            cpu_ticks: 0,
            io_bytes: 0,
            page_faults: 0,
            syscall_count: 0,
            recent_syscalls: [0; 8],
            syscall_idx: 0,
            quality: QualityScore {
                resource_efficiency: 1.0,
                causal_validity: 1.0,
                composite: 1.0,
            },
        }
    }

    /// 记录一个系统调用
    pub fn record_syscall(&mut self, sysno: u64) {
        self.recent_syscalls[self.syscall_idx as usize % 8] = sysno;
        self.syscall_idx = self.syscall_idx.wrapping_add(1);
        self.syscall_count = self.syscall_count.wrapping_add(1);
    }

    /// 获取最近的系统调用历史
    pub fn recent_syscalls_slice(&self) -> &[u64] {
        let n = self.syscall_idx.min(8) as usize;
        &self.recent_syscalls[..n]
    }
}

/// 固定大小进程表
pub struct ProcessTable {
    pub procs: [Process; MAX_PROCS],
    /// 下一个可用 PID
    next_pid: u16,
}

impl ProcessTable {
    pub fn new() -> Self {
        // 初始化所有槽位为 Free
        let procs: [Process; MAX_PROCS] = core::array::from_fn(|i| {
            let mut p = Process::new(i as u16);
            p.state = ProcState::Free;
            p
        });
        Self { procs, next_pid: 0 }
    }

    /// 分配新进程
    pub fn alloc(&mut self) -> Option<&mut Process> {
        for i in 0..MAX_PROCS {
            if self.procs[i].state == ProcState::Free {
                let pid = self.next_pid;
                self.next_pid = self.next_pid.wrapping_add(1);
                self.procs[i] = Process::new(pid);
                self.procs[i].pid = pid;
                self.procs[i].state = ProcState::Ready;
                return Some(&mut self.procs[i]);
            }
        }
        None
    }

    /// 按 PID 查找（只读）
    pub fn find(&self, pid: u16) -> Option<&Process> {
        for i in 0..MAX_PROCS {
            if self.procs[i].state != ProcState::Free && self.procs[i].pid == pid {
                return Some(&self.procs[i]);
            }
        }
        None
    }

    /// 按 PID 查找（可变）
    pub fn find_mut(&mut self, pid: u16) -> Option<&mut Process> {
        for i in 0..MAX_PROCS {
            if self.procs[i].state != ProcState::Free && self.procs[i].pid == pid {
                return Some(&mut self.procs[i]);
            }
        }
        None
    }

    /// 释放进程
    pub fn free(&mut self, pid: u16) {
        if let Some(proc) = self.find_mut(pid) {
            proc.state = ProcState::Free;
        }
    }

    /// 遍历所有活跃进程
    pub fn active_procs(&self) -> impl Iterator<Item = &Process> {
        self.procs.iter().filter(|p| p.state != ProcState::Free)
    }

    /// 可变的活跃进程遍历
    pub fn active_procs_mut(&mut self) -> impl Iterator<Item = &mut Process> {
        self.procs.iter_mut().filter(|p| p.state != ProcState::Free)
    }

    /// 活跃进程统计
    pub fn active_count(&self) -> usize {
        self.procs.iter().filter(|p| p.state != ProcState::Free).count()
    }
}
