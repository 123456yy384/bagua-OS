//! 动态八卦阵列 — 内核子系统动态互联拓扑
//!
//! 对应 AI 层：`DongTaiBaGuaZhen`
//!
//! OS 层主调度循环：每个调度周期——
//!   1. 收集各 slot 设备状态
//!   2. 生成极性向量
//!   3. 计算阻抗矩阵
//!   4. 根据阻抗分配 task quantum
//!   5. 审计 + 淘汰低效进程
//!   6. 更新 logic_pressure

use super::polarity::{ImpedanceMatrix, PolarityGenerator, PolarityVector, POLARITY_DIM};
use super::audit::ProcessAuditor;
use super::elimination::{EliminationAction, Eliminator};
use super::buffer::LoadSmoother;
use super::self_comp::HealthMonitor;
use super::process::{ProcessTable, ProcState, MAX_PROCS};
use super::task_gate::TaskPerceptor;
use crate::hal::device::DeviceState;

/// 八卦阵列调度器 — OS 主循环
pub struct BaguaScheduler {
    /// 各 slot 的设备状态
    pub slots: [DeviceState; 8],
    /// 各 slot 的极性向量
    polarities: [PolarityVector; 8],
    /// 当前阻抗矩阵
    pub impedance: ImpedanceMatrix,
    /// 负载平滑器
    smoother: LoadSmoother,
    /// 进程审计器
    auditor: ProcessAuditor,
    /// 淘汰器
    eliminator: Eliminator,
    /// 健康监控器
    pub health: HealthMonitor,
    /// 进程表
    pub proc_table: ProcessTable,
    /// 任务感知器
    #[allow(dead_code)]
    perceptor: TaskPerceptor,
    /// 当前 tick
    pub tick: u64,
    /// 各 slot 分配的 tick 配额（本轮）
    pub slot_quota: [u64; 8],
    /// 各 slot 存活分数（上一轮审计）
    pub slot_scores: [f32; 8],
}

impl BaguaScheduler {
    pub fn new(slots: [DeviceState; 8]) -> Self {
        Self {
            slots,
            polarities: [[0.0; POLARITY_DIM]; 8],
            impedance: ImpedanceMatrix::default(),
            smoother: LoadSmoother::new(),
            auditor: ProcessAuditor::new(100, 0.3),
            eliminator: Eliminator::new(0.3),
            health: HealthMonitor::new(),
            proc_table: ProcessTable::new(),
            perceptor: TaskPerceptor::new(),
            tick: 0,
            slot_quota: [0; 8],
            slot_scores: [0.5; 8],
        }
    }

    /// 主调度 tick（每个时钟中断调用一次）
    pub fn tick(&mut self) {
        self.tick += 1;

        // 1. 生成各 slot 的极性向量
        for i in 0..8 {
            self.polarities[i] = PolarityGenerator::generate(&self.slots[i]);
        }

        // 2. 计算阻抗矩阵
        for i in 0..8 {
            for j in 0..8 {
                if i != j {
                    let imp = PolarityGenerator::compute_impedance(
                        &self.polarities[i],
                        &self.polarities[j],
                    );
                    self.impedance.set(i, j, imp);
                }
            }
        }
        self.impedance.zero_diagonal();

        // 3. 根据阻抗分配资源
        self.allocate_resources();

        // 4. 审计进程
        if self.tick % self.auditor.audit_interval == 0 {
            self.run_audit();
        }

        // 5. 更新 logic_pressure
        self.slot_scores = self.collect_slot_scores();
        let pressure = self.health.update(&self.slot_scores);
        self.smoother.set_pressure(pressure);
    }

    /// 创建新进程
    pub fn spawn(&mut self, slot_id: u8) -> Option<u16> {
        if let Some(proc) = self.proc_table.alloc() {
            proc.slot_id = slot_id;
            let pid = proc.pid;
            Some(pid)
        } else {
            None
        }
    }

    /// 终止进程
    pub fn kill(&mut self, pid: u16) {
        self.proc_table.free(pid);
    }

    // ──── 内部实现 ────

    /// 根据阻抗矩阵分配各 slot 的计算资源配额
    ///
    /// 算法：
    ///   基础配额 = 总 tick / 活跃 slot 数
    ///   转移量 = sum_over_j( 基础配额_j / (1 + impedance[i][j]) ) × 0.1
    ///   最终配额 = 基础配额 + 转移量
    ///
    /// 阻抗越低 → 转移越多 → 配额越大（协同运算）
    /// 阻抗越高 → 转移越少 → 配额接近基础值（隔离）
    fn allocate_resources(&mut self) {
        let active_slots: usize = self.slots.iter()
            .filter(|s| s.load > 0.0 || s.class as u8 != 0)
            .count()
            .max(1);

        let base_quota: u64 = 1000 / active_slots as u64;

        for i in 0..8 {
            let mut quota = base_quota as f32;

            for j in 0..8 {
                if i != j {
                    let imp = self.impedance.get(i, j);
                    let transfer = base_quota as f32 / (1.0 + imp) * 0.1;
                    quota += transfer;
                }
            }

            self.slot_quota[i] = quota as u64;
        }
    }

    /// 运行一轮进程审计
    fn run_audit(&mut self) {
        let mut to_freeze: [u16; MAX_PROCS] = [0; MAX_PROCS];
        let mut freeze_count = 0;

        // 第一遍：评估质量，更新评分和 strikes
        for proc in self.proc_table.active_procs_mut() {
            let cpu = proc.cpu_ticks;
            let io = proc.io_bytes;
            let mem = proc.page_faults;

            let score = self.auditor.evaluate(proc.pid as u64, cpu, io, mem);
            proc.quality = score.clone();

            let action = self.eliminator.should_eliminate(score.composite, proc.audit_strikes);

            match action {
                EliminationAction::Keep => {
                    proc.audit_strikes = 0;
                }
                EliminationAction::Warn => {
                    proc.audit_strikes += 1;
                }
                EliminationAction::Freeze => {
                    if freeze_count < MAX_PROCS {
                        to_freeze[freeze_count] = proc.pid;
                        freeze_count += 1;
                    }
                }
            }
        }

        // 第二遍：执行冻结
        for i in 0..freeze_count {
            if let Some(proc) = self.proc_table.find_mut(to_freeze[i]) {
                proc.state = ProcState::Frozen;
                proc.audit_strikes = 0;
            }
        }
    }

    /// 收集各 slot 的存活分数
    ///
    /// 计算方式：
    ///   每个 slot 的存活率 = 该 slot 下非冻结进程的 quality.composite 平均值
    ///   无进程的 slot 默认 1.0
    fn collect_slot_scores(&self) -> [f32; 8] {
        let mut scores = [1.0f32; 8];
        let mut counts = [0u32; 8];
        let mut sums = [0.0f32; 8];

        for proc in self.proc_table.active_procs() {
            if proc.state != ProcState::Frozen {
                let s = proc.slot_id as usize;
                if s < 8 {
                    sums[s] += proc.quality.composite;
                    counts[s] += 1;
                }
            }
        }

        for i in 0..8 {
            if counts[i] > 0 {
                scores[i] = sums[i] / counts[i] as f32;
            }
            // 设备高负载时降低分数
            scores[i] *= 1.0 - self.slots[i].load * 0.5;
            scores[i] = scores[i].clamp(0.0, 1.0);
        }

        scores
    }

    /// 获取 slot 的当前极性向量
    pub fn polarity(&self, slot: usize) -> &PolarityVector {
        &self.polarities[slot]
    }
}
