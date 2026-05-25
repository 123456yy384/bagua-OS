//! 页表管理 — x86_64 4 级页表完整遍历
//!
//! 模拟模式：使用 Vec 存储页表节点，安全无 unsafe。
//! 裸机模式：替换为静态页帧池 + 物理地址。
//!
//! 架构：PML4 → PDPT → PD → PT → Page (4KB)

use crate::kernel::memory::JiuzhouAddress;

pub type PhysAddr = u64;
pub type VirtAddr = u64;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_TABLE_ENTRIES: usize = 512;

// ──── 页表条目 ────

#[derive(Clone, Copy, Default)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub const EMPTY: Self = Self(0);

    pub fn is_present(&self) -> bool  { self.0 & 1 != 0 }
    pub fn is_writable(&self) -> bool { self.0 & 2 != 0 }
    pub fn is_huge(&self) -> bool     { self.0 & 0x80 != 0 }

    pub fn address(&self) -> PhysAddr { self.0 & 0x000F_FFFF_FFFF_F000 }

    pub fn new(addr: PhysAddr, flags: u64) -> Self {
        Self((addr & 0x000F_FFFF_FFFF_F000) | (flags & 0xFFF))
    }

    pub fn present_writable(addr: PhysAddr) -> Self { Self::new(addr, 0x003) }
    pub fn present(addr: PhysAddr) -> Self          { Self::new(addr, 0x001) }
}

// ──── 页表节点（树形结构，Vec 存储子节点）────

struct PageTableNode {
    entries: [PageTableEntry; PAGE_TABLE_ENTRIES],
    /// 子节点索引（仅在 PML4/PDPT/PD 级别有效）
    children: Vec<PageTableNode>,
}

impl PageTableNode {
    fn new() -> Self {
        Self {
            entries: [PageTableEntry::EMPTY; PAGE_TABLE_ENTRIES],
            children: Vec::new(),
        }
    }
}

// ──── 页表管理器 ────

pub struct PageTableManager {
    root: PageTableNode,
    map_count: usize,
}

impl PageTableManager {
    pub fn new() -> Self {
        Self {
            root: PageTableNode::new(),
            map_count: 0,
        }
    }

    /// 映射虚拟页到物理页
    pub fn map(&mut self, vaddr: VirtAddr, paddr: PhysAddr, writable: bool) -> Result<Option<PhysAddr>, &'static str> {
        let pml4_idx = ((vaddr >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((vaddr >> 30) & 0x1FF) as usize;
        let pd_idx   = ((vaddr >> 21) & 0x1FF) as usize;
        let pt_idx   = ((vaddr >> 12) & 0x1FF) as usize;

        let node = Self::ensure_child(&mut self.root, pml4_idx);
        let node = Self::ensure_child(node, pdpt_idx);
        let node = Self::ensure_child(node, pd_idx);

        let entry = &mut node.entries[pt_idx];
        let old = if entry.is_present() { Some(entry.address()) } else { None };

        let flags = if writable { 0x003 } else { 0x001 };
        *entry = PageTableEntry::new(paddr, flags);
        self.map_count += 1;

        Ok(old)
    }

    /// 取消映射
    pub fn unmap(&mut self, vaddr: VirtAddr) -> Result<Option<PhysAddr>, &'static str> {
        let pml4_idx = ((vaddr >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((vaddr >> 30) & 0x1FF) as usize;
        let pd_idx   = ((vaddr >> 21) & 0x1FF) as usize;
        let pt_idx   = ((vaddr >> 12) & 0x1FF) as usize;

        let node = self.root.entries[pml4_idx];
        if !node.is_present() { return Ok(None); }
        let node = &mut self.root.children[pml4_idx];

        let entry = node.entries[pdpt_idx];
        if !entry.is_present() { return Ok(None); }
        let node = &mut node.children[pdpt_idx];

        let entry = node.entries[pd_idx];
        if !entry.is_present() { return Ok(None); }
        let node = &mut node.children[pd_idx];

        let entry = &mut node.entries[pt_idx];
        let old = if entry.is_present() { Some(entry.address()) } else { None };
        *entry = PageTableEntry::EMPTY;
        self.map_count = self.map_count.saturating_sub(1);

        Ok(old)
    }

    /// 虚拟地址 → 物理地址（4 级遍历）
    pub fn translate(&self, vaddr: VirtAddr) -> Option<PhysAddr> {
        let pml4_idx = ((vaddr >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((vaddr >> 30) & 0x1FF) as usize;
        let pd_idx   = ((vaddr >> 21) & 0x1FF) as usize;
        let pt_idx   = ((vaddr >> 12) & 0x1FF) as usize;
        let offset   = vaddr & 0xFFF;

        let e = self.root.entries[pml4_idx];
        if !e.is_present() { return None; }
        let n = &self.root.children[pml4_idx];

        let e = n.entries[pdpt_idx];
        if !e.is_present() { return None; }
        let n = &n.children[pdpt_idx];

        let e = n.entries[pd_idx];
        if !e.is_present() { return None; }
        // 注：当前不使用 huge page bit，走完整 PT 路径
        let n = &n.children[pd_idx];

        let e = n.entries[pt_idx];
        if !e.is_present() { return None; }

        Some(e.address() + offset)
    }

    /// 九州地址 → 物理地址
    pub fn map_jiuzhou(&self, addr: &JiuzhouAddress) -> Option<PhysAddr> {
        self.translate(addr.to_physical())
    }

    pub fn map_count(&self) -> usize { self.map_count }

    // ──── 内部 ────

    /// 确保子节点存在（不存在则创建）
    fn ensure_child<'a>(parent: &'a mut PageTableNode, idx: usize) -> &'a mut PageTableNode {
        if parent.entries[idx].is_present() {
            // 已存在，返回已有子节点
            &mut parent.children[idx]
        } else {
            // 不存在，创建新节点
            let child = PageTableNode::new();
            if parent.children.len() <= idx {
                parent.children.resize_with(idx + 1, PageTableNode::new);
            }
            parent.children[idx] = child;
            parent.entries[idx] = PageTableEntry::present(0x1000); // 占位物理地址
            // 注：在真实 MMU 中 entries[idx] 存储子表物理地址；
            // 模拟模式下用 Vec 索引，entries[idx] 仅作为 present 标记
            &mut parent.children[idx]
        }
    }
}
