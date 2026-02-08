// Monolithic Wave-State Operator (MWSO) - QL-RF Evolution
// 512 parameters, complex latent space, PDE-based intelligence.

use std::f32::consts::PI;

pub struct MWSO {
    pub psi_real: Vec<f32>,
    pub psi_imag: Vec<f32>,
    pub theta: Vec<f32>,
    pub frequencies: Vec<f32>,
    pub dim: usize,
}

impl MWSO {
    pub fn new() -> Self {
        let dim = 256;
        let mut theta = vec![0.0; 512];
        let mut frequencies = vec![0.0; dim];
        for i in 0..512 { theta[i] = (i as f32 * 0.1).sin() * 0.1; }
        for i in 0..dim { frequencies[i] = (i as f32 / dim as f32).powi(2) * 2.0 * PI; }

        let mut psi_real = vec![0.0; dim];
        // 全ての成分に微弱な初期エネルギーを分布させる（真っさらな水面）
        for i in 0..dim { psi_real[i] = 0.01; }

        Self { psi_real, psi_imag: vec![0.0; dim], theta, frequencies, dim }
    }

    pub fn step(&mut self, input: &[f32], dt: f32) {
        // 1. External Wave Injection
        for (i, &val) in input.iter().enumerate() {
            if i < self.dim {
                // thetaを増幅率として入力
                self.psi_real[i] += val * self.theta[i % 512].abs();
            }
        }

        // 2. Wave Propagation
        for i in 0..self.dim {
            let omega = self.frequencies[i];
            let re = self.psi_real[i];
            let im = self.psi_imag[i];
            
            let cos_w = (omega * dt).cos();
            let sin_w = (omega * dt).sin();
            let mut new_re = re * cos_w - im * sin_w;
            let mut new_im = re * sin_w + im * cos_w;

            // 隣接・遠隔干渉
            let coupling_idx = i % 512;
            let coupling_strength = self.theta[coupling_idx];
            let next_idx = (i + 1) % self.dim;
            let prev_idx = if i == 0 { self.dim - 1 } else { i - 1 };
            
            let local_interaction = coupling_strength * (self.psi_real[next_idx] + self.psi_real[prev_idx]);
            let stride = (self.theta[(i + 128) % 512].abs() * 128.0) as usize % self.dim;
            let remote_idx = (i + stride) % self.dim;
            let remote_interaction = self.theta[(i + 384) % 512] * self.psi_real[remote_idx];
            
            new_re += (local_interaction + remote_interaction) * dt;
            
            // 液体の粘性による減衰 (Liquid Memory)
            let viscosity = 0.01 * (1.0 - self.theta[(i + 256) % 512].abs());
            new_re *= 1.0 - viscosity;
            new_im *= 1.0 - viscosity;

            self.psi_real[i] = new_re;
            self.psi_imag[i] = new_im;
        }

        self.normalize();
    }

    fn normalize(&mut self) {
        let mut total_energy = 0.0;
        for i in 0..self.dim {
            total_energy += self.psi_real[i].powi(2) + self.psi_imag[i].powi(2);
        }
        
        let norm = total_energy.sqrt();
        if norm > 1e-6 {
            for i in 0..self.dim {
                self.psi_real[i] /= norm;
                self.psi_imag[i] /= norm;
            }
        }
    }

    pub fn get_action_scores(&self, offset: usize, size: usize) -> Vec<f32> {
        let mut scores = Vec::with_capacity(size);
        for i in 0..size {
            let mut score = 0.0;
            for j in 0..16 {
                let psi_idx = (offset + i * 16 + j) % self.dim;
                let theta_idx = (offset + i * 16 + j) % 512;
                
                let re = self.psi_real[psi_idx];
                let im = self.psi_imag[psi_idx];
                let phase = im.atan2(re);
                
                // 共鳴スコアリング: 位相の同調度
                score += (re.powi(2) + im.powi(2)).sqrt() * (phase - self.theta[theta_idx]).cos();
            }
            scores.push(score);
        }
        scores
    }

    /// QL-RF 位相同期学習
    pub fn adapt(&mut self, reward: f32, last_actions: &[usize]) {
        let lr = 0.1; // 学習率を大幅に強化
        for &action_idx in last_actions {
            for j in 0..16 {
                let theta_idx = (action_idx * 16 + j) % 512;
                let psi_idx = (action_idx * 16 + j) % self.dim;
                
                let re = self.psi_real[psi_idx];
                let im = self.psi_imag[psi_idx];
                let current_phase = im.atan2(re);
                
                // 報酬に応じた位相の引き込み
                let target_phase = if reward > 0.0 { 0.0 } else { PI };
                let phase_diff = target_phase - current_phase;

                // theta（屈折率/位相フィルタ）を更新
                self.theta[theta_idx] += phase_diff.sin() * lr * reward.abs();
                self.theta[theta_idx] = self.theta[theta_idx].clamp(-PI, PI);
                
                // 粘性（記憶持続）の更新
                let v_idx = (theta_idx + 256) % 512;
                self.theta[v_idx] = (self.theta[v_idx] + reward * 0.1).clamp(-1.0, 1.0);
            }
        }
    }

    pub fn calculate_rhyd(&self) -> f32 {
        let mut rd = 0.0;
        let mut active_components = 0.0;
        for i in 0..self.dim {
            let energy = self.psi_real[i].powi(2) + self.psi_imag[i].powi(2);
            let phase = self.psi_imag[i].atan2(self.psi_real[i]);
            
            // 位相が目標(0)にどれだけ近いか
            let sync = (phase.cos() + 1.0) / 2.0; 
            if energy > 0.001 {
                rd += energy * sync;
                active_components += 1.0;
            }
        }
        // エネルギーの総和 × 活性化コンポーネント数による「知能の広がり」
        rd * (active_components / 256.0) * 100.0
    }
}
