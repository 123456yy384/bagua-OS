//! 卦象对冲 — 极性驱动调度器
//!
//! 对应 AI 层：`GuaXiangDuiChong`
//!
//! 核心公式：
//!   polarity_i = extend(capability_vector_i, load, temp, power)
//!   dot[i][j] = polarity_i · polarity_j
//!   impedance[i][j] = softplus(dot[i][j])
//!
//! OS 层极性向量生成方案：
//!   前 8 维：设备 capability_vector（硬编码，来自 DeviceClass）
//!   中 8 维：运行时状态（负载、温度、功耗 × 2 + 填充）
//!   后 16 维：谐波编码（前 8 维的 sin/cos 变换，增加非线性分离度）
//!
//!   总共 32 维，计算方式固定，无学习参数。

use crate::hal::device::DeviceState;

pub const POLARITY_DIM: usize = 32;
pub const CAPABILITY_DIM: usize = 8;

/// 极性向量（32 维）
pub type PolarityVector = [f32; POLARITY_DIM];

/// 8×8 阻抗矩阵
#[derive(Clone)]
pub struct ImpedanceMatrix {
    pub data: [[f32; 8]; 8],
}

impl Default for ImpedanceMatrix {
    fn default() -> Self {
        Self {
            data: [[1.0; 8]; 8], // 默认高阻抗（隔离）
        }
    }
}

impl ImpedanceMatrix {
    pub fn get(&self, i: usize, j: usize) -> f32 {
        self.data[i][j]
    }

    pub fn set(&mut self, i: usize, j: usize, val: f32) {
        self.data[i][j] = val;
    }

    /// 对角线置零（自身不对自身施加阻抗）
    pub fn zero_diagonal(&mut self) {
        for i in 0..8 {
            self.data[i][i] = 0.0;
        }
    }
}

/// 极性生成器
pub struct PolarityGenerator;

impl PolarityGenerator {
    /// 从设备状态生成 32 维极性向量
    ///
    /// 布局：
    ///   [0..8)   = capability_vector（硬件特征，8 维）
    ///   [8..16)  = runtime_state（负载、温度、功耗，重复填充）
    ///   [16..32) = harmonics（前 8 维的 sin/cos 非线性变换）
    pub fn generate(state: &DeviceState) -> PolarityVector {
        let cap = state.class.capability_vector();
        let mut polarity = [0.0f32; POLARITY_DIM];

        // 前 8 维：硬件能力特征
        polarity[0..8].copy_from_slice(&cap);

        // 中 8 维：运行时状态
        polarity[8] = state.load.clamp(0.0, 1.0);
        polarity[9] = (state.temperature / 120.0).clamp(0.0, 1.0);
        polarity[10] = (state.power_draw / 500.0).clamp(0.0, 1.0);
        // 重复填充使向量更均衡
        polarity[11] = state.load.clamp(0.0, 1.0) * 0.5;
        polarity[12] = (state.temperature / 120.0).clamp(0.0, 1.0) * 0.5;
        polarity[13] = state.load.clamp(0.0, 1.0) * 0.25;
        polarity[14] = 1.0 - state.load.clamp(0.0, 1.0);
        polarity[15] = 0.0;

        // 后 16 维：谐波编码（前 8 维的 sin/cos）
        // 增加非线性分离度，使不同设备的极性差异更显著
        use core::f32::consts::PI;
        for i in 0..8 {
            let phase = PI * (i as f32 + 1.0) / 4.0;
            polarity[16 + i] = libm::sinf(cap[i] * phase);
            polarity[24 + i] = libm::cosf(cap[i] * phase);
        }

        polarity
    }

    /// 计算两个极性向量的点积
    pub fn dot_product(a: &PolarityVector, b: &PolarityVector) -> f32 {
        let mut dot = 0.0f32;
        for i in 0..POLARITY_DIM {
            dot += a[i] * b[i];
        }
        dot
    }

    /// 计算阻抗：softplus(dot_product)
    /// softplus(x) = ln(1 + e^x)，保证输出为正
    pub fn compute_impedance(a: &PolarityVector, b: &PolarityVector) -> f32 {
        let dot = Self::dot_product(a, b);
        if dot > 20.0 {
            dot
        } else {
            libm::log1pf(libm::expf(dot))
        }
    }
}
