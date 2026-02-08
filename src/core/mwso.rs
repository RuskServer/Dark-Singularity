// Monolithic Wave-State Operator (MWSO) - Elastic Evolution
// Analog Penalty Fields, Dissipative Failure Memory.

use std::f32::consts::PI;

pub struct MWSO {
    pub psi_real: Vec<f32>,
    pub psi_imag: Vec<f32>,
    pub theta: Vec<f32>,
    pub frequencies: Vec<f32>,
    pub dim: usize,
}

impl MWSO {
    pub fn new(dim: usize) -> Self {
        let theta_size = dim * 2;
        let mut theta = vec![0.0; theta_size];
        let mut frequencies = vec![0.0; dim];
        for i in 0..theta_size { theta[i] = (i as f32 * 0.1).sin() * 0.1; }
        for i in 0..dim { frequencies[i] = (i as f32 / dim as f32).powi(2) * 2.0 * PI; }
        Self { psi_real: vec![0.01; dim], psi_imag: vec![0.0; dim], theta, frequencies, dim }
    }

    pub fn inject_state(&mut self, state_idx: usize, strength: f32, penalty_field: &[f32]) {
        if state_idx >= self.dim { return; }
        let primes = [31, 37, 41, 43, 47, 53, 59, 61, 67, 71];
        let stride = primes[state_idx % primes.len()];
        let phase_offset = (state_idx as f32 * 1.618).rem_euclid(2.0 * PI);
        
        for i in 0..16 { 
            let idx = (state_idx + i * stride) % self.dim;
            
            // ペナルティ（反発力）がある領域は、注入強度を弱める
            let penalty = penalty_field.get(idx).cloned().unwrap_or(0.0);
            let resistance = (-penalty * 2.0).exp(); // ペナルティが高いほど抵抗が強まる
            
            let phase_filter = self.theta[idx].cos() + phase_offset;
            let drive = strength * (1.5 + phase_filter.cos()) * resistance;
            self.psi_real[idx] += drive;
            self.psi_imag[idx] += drive * phase_filter.sin();
        }
    }

    pub fn step_core(&mut self, dt: f32, speed_boost: f32, focus_factor: f32, system_temp: f32) {
        let solidification = 0.9999 - (0.0005 * (1.0 - focus_factor));
        let effective_dt = dt * (1.0 + speed_boost);

        for i in 0..self.dim {
            self.theta[i] *= solidification;
            self.theta[i + self.dim] *= solidification;

            let omega = self.frequencies[i];
            let (re, im) = (self.psi_real[i], self.psi_imag[i]);
            let (sin_w, cos_w) = (omega * effective_dt).sin_cos();
            
            let new_re = re * cos_w - im * sin_w;
            let new_im = re * sin_w + im * cos_w;

            let coupling_strength = self.theta[i];
            let next_idx = (i + 1) % self.dim;
            let prev_idx = if i == 0 { self.dim - 1 } else { i - 1 };
            
            let resonance = coupling_strength * (self.psi_real[next_idx] + self.psi_real[prev_idx]);
            self.psi_real[i] = new_re + resonance * effective_dt * (1.0 + focus_factor);
            self.psi_imag[i] = new_im;
            
            let viscosity = 0.01 * (1.1 - self.theta[i + self.dim].clamp(-1.0, 1.0).abs());
            self.psi_real[i] *= 1.0 - viscosity;
            self.psi_imag[i] *= 1.0 - viscosity;
        }
        let target_norm = 1.0 + (system_temp * 0.5).min(1.5);
        self.normalize(target_norm);
    }

    fn normalize(&mut self, target_norm: f32) {
        let mut total_energy_sq = 0.0;
        for i in 0..self.dim { total_energy_sq += self.psi_real[i].powi(2) + self.psi_imag[i].powi(2); }
        let norm = total_energy_sq.sqrt();
        if norm > 1e-6 {
            let factor = target_norm / norm;
            for i in 0..self.dim { self.psi_real[i] *= factor; self.psi_imag[i] *= factor; }
        }
    }

    pub fn get_action_scores(&self, offset: usize, size: usize, exploration_noise: f32, penalty_field: &[f32]) -> Vec<f32> {
        let bin_per_action = self.dim / size;
        let mut scores = Vec::with_capacity(size);
        for i in 0..size {
            let mut score = 0.0;
            let center_idx = (offset + i * bin_per_action) % self.dim;
            let mut total_penalty = 0.0;

            for j in 0..bin_per_action { 
                let idx = (center_idx + j) % self.dim;
                let (re, im) = (self.psi_real[idx], self.psi_imag[idx]);
                score += (re.powi(2) + im.powi(2)).sqrt() * (im.atan2(re) - self.theta[idx]).cos();
                total_penalty += penalty_field.get(idx).cloned().unwrap_or(0.0);
            }

            // ペナルティをスコアから減算（ハード・マスキングではなくソフトな反発力）
            score -= total_penalty * 0.5;
            
            score = (score * 1.5).exp().min(1e10);
            if exploration_noise > 0.0 {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
                score += (((seed + i as u128) % 1000) as f32 / 1000.0 - 0.5) * exploration_noise;
            }
            scores.push(score);
        }
        scores
    }

    pub fn adapt(&mut self, reward: f32, last_actions: &[usize], system_temp: f32, action_size: usize) {
        let annealing = (system_temp * 0.5).clamp(0.1, 1.0);
        let base_lr = 1.2 * annealing; 
        let bin_per_action = self.dim / action_size;
        let t_len = self.theta.len();

        for &action_idx in last_actions {
            if reward < 0.0 {
                let base = (action_idx * bin_per_action) % self.dim;
                for j in 0..bin_per_action {
                    let idx = (base + j) % self.dim;
                    self.frequencies[idx] = (self.frequencies[idx] + 0.001).clamp(0.0, 2.0 * PI);
                }
            }
            for neighborhood in -1..=1 {
                let weight = if neighborhood == 0 { 1.0 } else { 0.2 };
                let target_action = (action_idx as i32 + neighborhood).rem_euclid(action_size as i32) as usize;
                let lr = base_lr * weight;
                let base_idx = target_action * bin_per_action;
                for j in 0..bin_per_action {
                    let idx = (base_idx + j) % self.dim;
                    let current_phase = self.psi_imag[idx].atan2(self.psi_real[idx]);
                    let target_phase = if reward > 0.0 { 0.0 } else { PI };
                    let phase_diff_sin = (target_phase - current_phase).sin();
                    self.theta[idx] = (self.theta[idx] + phase_diff_sin * lr).clamp(-PI, PI);
                    if reward > 0.0 {
                        let (sin_p, cos_p) = current_phase.sin_cos();
                        self.psi_real[idx] += 1.5 * reward * cos_p;
                        self.psi_imag[idx] += 1.5 * reward * sin_p;
                        self.theta[(idx + self.dim) % t_len] = 1.0; 
                    }
                }
            }
        }
    }

    pub fn inject_exploration_noise(&mut self, strength: f32) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        for i in 0..self.dim {
            let noise = ((seed % (i as u128 + 1)) as f32 / (i as f32 + 1.0)).sin();
            self.psi_real[i] += noise * strength;
        }
    }

    pub fn calculate_rhyd(&self) -> f32 {
        let mut rd = 0.0;
        let mut active_components = 0.0;
        for i in 0..self.dim {
            let energy_sq = self.psi_real[i].powi(2) + self.psi_imag[i].powi(2);
            if energy_sq > 0.001 {
                let phase = self.psi_imag[i].atan2(self.psi_real[i]);
                rd += energy_sq * (phase.cos() + 1.0) / 2.0;
                active_components += 1.0;
            }
        }
        rd * (active_components / self.dim as f32) * 100.0
    }
}
