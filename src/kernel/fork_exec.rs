//! fork/exec 实现
//!
//! 两种模式：
//!   - COW (默认)：Copy-On-Write fork，成熟方案
//!   - FORK_CLEAN：激进隔离，子进程不继承内存，通过消息获取上下文

use crate::kernel::process::{ProcessTable, ProcState};
use crate::kernel::pagetable::PageTableManager;

/// fork 结果
pub enum ForkResult {
    /// 父进程拿到子进程 PID
    Parent(u16),
    /// 子进程
    Child,
}

/// 进程创建器 — 集成进程表和页表
pub struct ProcessSpawner {
    proc_table: ProcessTable,
    page_mgr: PageTableManager,
}

impl ProcessSpawner {
    pub fn new() -> Self {
        Self {
            proc_table: ProcessTable::new(),
            page_mgr: PageTableManager::new(),
        }
    }

    /// COW fork：复制页表（只读共享，写时复制）
    ///
    /// 当前实现：创建新进程，标记为 COW 模式。
    /// 实际 COW 需要页表级别的写保护 + 缺页异常处理。
    pub fn fork_cow(&mut self, parent_pid: u16) -> Result<ForkResult, &'static str> {
        // 验证父进程存在
        if self.proc_table.find(parent_pid).is_none() {
            return Err("parent process not found");
        }

        // 分配新 PCB
        let child = self.proc_table.alloc()
            .ok_or("process table full")?;
        let child_pid = child.pid;

        // 保留父进程信息（避免 borrow 冲突）
        let parent_slot = self.proc_table.find(parent_pid)
            .map(|p| p.slot_id)
            .unwrap_or(0);

        // 初始化子进程
        if let Some(child) = self.proc_table.find_mut(child_pid) {
            child.slot_id = parent_slot;
            child.state = ProcState::Ready;
            // COW 模式下，子进程共享父进程页表（只读）
            // 实际实现需要 page_mgr.cow_clone(parent_page_table)
        }

        Ok(ForkResult::Parent(child_pid))
    }

    /// FORK_CLEAN fork：零共享内存，通过消息传递上下文
    pub fn fork_clean(&mut self, _parent_pid: u16, _init_msg: &[u8]) -> Result<ForkResult, &'static str> {
        // 分配新 PCB
        let child = self.proc_table.alloc()
            .ok_or("process table full")?;
        let child_pid = child.pid;

        // 子进程不继承任何内存
        // 通过 _init_msg 获取初始数据
        if let Some(child) = self.proc_table.find_mut(child_pid) {
            child.state = ProcState::Ready;
            child.slot_id = 0;
        }

        Ok(ForkResult::Parent(child_pid))
    }

    /// exec：替换进程映像
    ///
    /// 加载新的程序，替换当前进程的地址空间。
    /// 当前骨架：清理旧映射，加载新页表。
    pub fn exec(&mut self, _pid: u16, _program: &[u8]) -> Result<(), &'static str> {
        // TODO: 1. 解析 ELF
        // TODO: 2. 清理旧页表映射
        // TODO: 3. 建立新映射（text/data/bss/stack）
        // TODO: 4. 设置入口点
        Ok(())
    }

    /// 回收进程资源
    pub fn reap(&mut self, pid: u16) {
        // TODO: 释放进程的所有页面映射
        self.proc_table.free(pid);
    }

    // 访问器
    pub fn proc_table(&self) -> &ProcessTable { &self.proc_table }
    pub fn proc_table_mut(&mut self) -> &mut ProcessTable { &mut self.proc_table }
    pub fn page_mgr(&self) -> &PageTableManager { &self.page_mgr }
}
