// Monolithic Wave-State Operator (MWSO) - Elastic Evolution
// Analog Penalty Fields, Dissipative Failure Memory.

use std::f32::consts::PI;

pub struct MWSO {
    pub psi_real: Vec<f32>,
    pub psi_imag: Vec<f32>,
    pub theta: Vec<f32>,
    pub frequencies: Vec<f32>,
    pub gravity_field: Vec<f32>, 
    pub entanglements: Vec<(usize, usize, f32)>, 
    
    // --- Global Memory Wave (Quantum Superposition) ---
    // A single wave that stores multiple experiences through interference patterns.
    pub memory_psi_real: Vec<f64>,
    pub memory_psi_imag: Vec<f64>,
    
    pub dim: usize,
}

impl MWSO {
    pub fn new(dim: usize) -> Self {
        let theta_size = dim * 2;
        let mut theta = vec![0.0; theta_size];
        let mut frequencies = vec![0.0; dim];
        for i in 0..theta_size { theta[i] = (i as f32 * 0.1).sin() * 0.1; }
        for i in 0..dim { frequencies[i] = (i as f32 / dim as f32).powi(2) * 2.0 * PI; }
        Self { 
            psi_real: vec![0.01; dim], 
            psi_imag: vec![0.0; dim], 
            theta, 
            frequencies, 
            gravity_field: vec![0.0; dim],
            entanglements: Vec::new(),
            memory_psi_real: vec![0.0; dim],
            memory_psi_imag: vec![0.0; dim],
            dim 
        }
    }

    pub fn add_wormhole(&mut self, from: usize, to: usize, strength: f32) {
        if from < self.dim && to < self.dim {
            self.entanglements.push((from, to, strength));
        }
    }

    /// Imprints a state into the global memory wave via superposition.
    pub fn imprint_memory(&mut self, psi_real: &[f32], psi_imag: &[f32], strength: f32) {
        if psi_real.len() != self.dim || psi_imag.len() != self.dim { return; }
        for i in 0..self.dim {
            self.memory_psi_real[i] += psi_real[i] as f64 * strength as f64;
            self.memory_psi_imag[i] += psi_imag[i] as f64 * strength as f64;
        }
        // 次元数に比例した正規化
        let target = self.dim as f64 * 0.01;
        self.normalize_memory(target);
    }

    fn normalize_memory(&mut self, target_norm: f64) {
        let mut total_energy_sq = 0.0;
        for i in 0..self.dim { total_energy_sq += self.memory_psi_real[i].powi(2) + self.memory_psi_imag[i].powi(2); }
        let norm = total_energy_sq.sqrt();
        if norm > 1e-12 {
            let factor = target_norm / norm;
            for i in 0..self.dim { self.memory_psi_real[i] *= factor; self.memory_psi_imag[i] *= factor; }
        }
    }

    pub fn inject_state(&mut self, state_idx: usize, strength: f32, penalty_field: &[f32]) {
        if state_idx >= self.dim { return; }
        let primes = [31, 37, 41, 43, 47, 53, 59, 61, 67, 71];
        let stride = primes[state_idx % primes.len()];
        let phase_offset = (state_idx as f32 * 1.618).rem_euclid(2.0 * PI);
        
        for i in 0..16 { 
            let idx = (state_idx + i * stride) % self.dim;
            
            let penalty = penalty_field.get(idx).cloned().unwrap_or(0.0);
            let resistance = (-penalty * 2.0).exp(); 
            
            let phase_filter = self.theta[idx].cos() + phase_offset;
            let drive = strength * (1.5 + phase_filter.cos()) * resistance;
            self.psi_real[idx] += drive;
            self.psi_imag[idx] += drive * phase_filter.sin();
        }
    }

    pub fn step_core(&mut self, dt: f32, speed_boost: f32, focus_factor: f32, system_temp: f32, penalty_field: &[f32]) {
        let solidification = 0.9999 - (0.0005 * (1.0 - focus_factor));
        let effective_dt = dt * (1.0 + speed_boost);

        // Calculate overlap (resonance) with the memory wave
        let mut overlap_re = 0.0_f64;
        let mut overlap_im = 0.0_f64;
        for i in 0..self.dim {
            overlap_re += self.psi_real[i] as f64 * self.memory_psi_real[i] + self.psi_imag[i] as f64 * self.memory_psi_imag[i];
            overlap_im += self.psi_real[i] as f64 * self.memory_psi_imag[i] - self.psi_imag[i] as f64 * self.memory_psi_real[i];
        }
        let resonance_amplitude = (overlap_re.powi(2) + overlap_im.powi(2)).sqrt().min(1.0) as f32;

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
            
            let coupling_resonance = coupling_strength * (self.psi_real[next_idx] + self.psi_real[prev_idx]);
            
            // --- Memory Interaction ---
            // If the current state resonates with the memory wave, it flows into the active state.
            // This is "Quantum Mechanical Reminiscence".
            let memory_flow_re = (self.memory_psi_real[i] * resonance_amplitude as f64 * 0.5) as f32;
            let memory_flow_im = (self.memory_psi_imag[i] * resonance_amplitude as f64 * 0.5) as f32;

            self.psi_real[i] = new_re + (coupling_resonance + memory_flow_re) * effective_dt * (1.0 + focus_factor);
            self.psi_imag[i] = new_im + memory_flow_im * effective_dt * (1.0 + focus_factor);
            
            // 重力場による「事象の地平線」効果：重力が強いほど忘却（粘性）が消える
            let gravity = self.gravity_field[i];
            let penalty = penalty_field.get(i).cloned().unwrap_or(0.0);
            
            let base_viscosity = 0.01 * (1.1 - self.theta[i + self.dim].clamp(-1.0, 1.0).abs());
            // ペナルティ場による強制減衰（極端なペナルティ場）
            let viscosity = base_viscosity * (1.0 - gravity).max(0.001) + penalty * 0.5; 

            self.psi_real[i] *= (1.0 - viscosity * effective_dt).max(0.0);
            self.psi_imag[i] *= (1.0 - viscosity * effective_dt).max(0.0);
        }

        // ワームホールによる量子もつれ（位相の同期）
        for &(a, b, strength) in &self.entanglements {
            let p1_real = self.psi_real[a];
            let p1_imag = self.psi_imag[a];
            self.psi_real[b] += p1_real * strength * effective_dt;
            self.psi_imag[b] += p1_imag * strength * effective_dt;
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
            let base_idx = action_idx * bin_per_action;

            if reward > 2.0 {
                // 強力な報酬：重力場を形成（ブラックホール化）
                for j in 0..bin_per_action {
                    let idx = (base_idx + j) % self.dim;
                    self.gravity_field[idx] = (self.gravity_field[idx] + 0.1).min(1.0);
                }
                
                // --- Imprint Memory on Success ---
                // If a great result is achieved, imprint the current state into the global memory wave.
                let psi_re = self.psi_real.clone();
                let psi_im = self.psi_imag.clone();
                self.imprint_memory(&psi_re, &psi_im, reward * 0.2);
            }

            if reward < 0.0 {
                for j in 0..bin_per_action {
                    let idx = (base_idx + j) % self.dim;
                    self.frequencies[idx] = (self.frequencies[idx] + 0.001).clamp(0.0, 2.0 * PI);
                    self.gravity_field[idx] *= 0.8; // 失敗は重力を弱める
                }
            }
            for neighborhood in -1..=1 {
                let weight = if neighborhood == 0 { 1.0 } else { 0.2 };
                let target_action = (action_idx as i32 + neighborhood).rem_euclid(action_size as i32) as usize;
                let lr = base_lr * weight;
                let n_base = target_action * bin_per_action;
                for j in 0..bin_per_action {
                    let idx = (n_base + j) % self.dim;
                    let current_phase = self.psi_imag[idx].atan2(self.psi_real[idx]);
                    let target_phase = if reward > 0.0 { 0.0 } else { PI };
                    let phase_diff_sin = (target_phase - current_phase).sin();
                    
                    // 重力が強い場所は、位相が「固定」されやすくなる
                    let gravity_inertia = 1.0 - self.gravity_field[idx] * 0.5;
                    self.theta[idx] = (self.theta[idx] + phase_diff_sin * lr * gravity_inertia).clamp(-PI, PI);
                    
                    if reward > 0.0 {
                        let (sin_p, cos_p) = current_phase.sin_cos();
                        self.psi_real[idx] += 1.5 * reward * cos_p;
                        self.psi_imag[idx] += 1.5 * reward * sin_p;
                        self.theta[(idx + self.dim) % t_len] = 1.0; 
                    }
                }
            }
        }

        // ホーキング放射（重力場の自然蒸発）
        for g in &mut self.gravity_field { *g *= 0.999; }
    }

    /// 行動から動機を逆算するための位相アライメント
    pub fn align_to_action(&mut self, action_idx: usize, strength: f32, action_size: usize) {
        let bin_per_action = self.dim / action_size;
        let base_idx = (action_idx * bin_per_action) % self.dim;
        let lr = 0.5 * strength;

        for j in 0..bin_per_action {
            let idx = (base_idx + j) % self.dim;
            let current_phase = self.psi_imag[idx].atan2(self.psi_real[idx]);
            let target_phase = 0.0;
            let phase_diff_sin = (target_phase - current_phase).sin();
            self.theta[idx] = (self.theta[idx] + phase_diff_sin * lr).clamp(-PI, PI);
            self.psi_real[idx] += 0.2 * strength;
            self.gravity_field[idx] = (self.gravity_field[idx] + 0.01 * strength).min(0.5);
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

    pub fn inject_external_state(&mut self, psi_real: &[f32], psi_imag: &[f32], strength: f32) {
        if psi_real.len() != self.dim || psi_imag.len() != self.dim { return; }
        for i in 0..self.dim {
            self.psi_real[i] += psi_real[i] * strength;
            self.psi_imag[i] += psi_imag[i] * strength;
        }
    }

    pub fn calculate_rhyd(&self) -> f32 {
        let mut rd = 0.0;
        for i in 0..self.dim {
            let energy_sq = self.psi_real[i].powi(2) + self.psi_imag[i].powi(2);
            if energy_sq > 0.001 {
                let phase = self.psi_imag[i].atan2(self.psi_real[i]);
                rd += energy_sq * (phase.cos() + 1.0) / 2.0;
            }
        }
        rd * 100.0 / self.dim as f32  // dimで1回だけ割る
    }
}
