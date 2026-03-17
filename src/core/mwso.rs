// Monolithic Wave-State Operator (MWSO) - Elastic Evolution
// Analog Penalty Fields, Dissipative Failure Memory.

use std::collections::HashMap;
use std::f32::consts::PI;

pub struct MWSO {
    pub psi_real: Vec<f32>,
    pub psi_imag: Vec<f32>,
    pub theta: Vec<f32>,
    pub frequencies: Vec<f32>,
    pub gravity_field: Vec<f32>, 
    pub entanglements: Vec<(usize, usize, f32)>, 
    
    // --- PP-CEL: Pure-Phase Correlated Energy Landscape ---
    pub q_memory_re: Vec<f64>,
    pub q_memory_im: Vec<f64>,
    pub q_topo_re: Vec<f64>,   // Topological correlation (Gradients)
    pub q_topo_im: Vec<f64>,
    pub energy_landscape: Vec<f32>, // Dynamic potential field (V)
    pub input_signature: Vec<f32>,  // Quantized current input (Query)
    
    pub scramble_phases: Vec<f32>,
    
    pub dim: usize,
    pub rng_seed: u64,
}

impl MWSO {
    pub fn new(dim: usize) -> Self {
        let theta_size = dim * 2;
        let mut theta = vec![0.0; theta_size];
        let mut frequencies = vec![0.0; dim];
        for i in 0..theta_size { theta[i] = (i as f32 * 0.1).sin() * 0.1; }
        for i in 0..dim { frequencies[i] = (i as f32 / dim as f32).powi(2) * 2.0 * PI; }
        
        let mut scramble_phases = vec![0.0; dim];
        for i in 0..dim {
            // Deterministic random phases based on Golden Ratio
            scramble_phases[i] = (i as f32 * 1.61803398875).rem_euclid(2.0 * PI);
        }

        let mut entanglements = Vec::new();
        // --- Small-World Network Construction ---
        // Add 5% random long-range connections (wormholes)
        let wormhole_count = (dim as f32 * 0.05) as usize;
        let mut seed = 0x12345678u64; // Local seed for construction
        for _ in 0..wormhole_count {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let from = (seed % dim as u64) as usize;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let to = (seed % dim as u64) as usize;
            if from != to {
                entanglements.push((from, to, 0.05)); // Slight coupling
            }
        }

        Self { 
            psi_real: vec![0.01; dim], 
            psi_imag: vec![0.0; dim], 
            theta, 
            frequencies, 
            gravity_field: vec![0.0; dim],
            entanglements,
            q_memory_re: vec![0.0; dim],
            q_memory_im: vec![0.0; dim],
            q_topo_re: vec![0.0; dim],
            q_topo_im: vec![0.0; dim],
            energy_landscape: vec![0.0; dim],
            input_signature: vec![0.0; dim],
            scramble_phases,
            dim,
            rng_seed: 0xDEADBEEF,
        }
    }

    pub fn next_rng(&mut self) -> f32 {
        self.rng_seed = self.rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.rng_seed >> 32) as u32) as f32 / u32::MAX as f32
    }

    pub fn add_wormhole(&mut self, from: usize, to: usize, strength: f32) {
        if from < self.dim && to < self.dim {
            self.entanglements.push((from, to, strength));
        }
    }

    /// PP-CEL: Pure-Phase Correlated Energy Landscape Imprinting.
    /// Uses pure phase correlations weighted by reward (alpha) with normalization.
    pub fn imprint_qcel(&mut self, input_idx: usize, reward: f32) {
        // Alpha: Weight of the memory update (Reward/Confidence)
        let alpha = reward.max(0.1) as f64;
        let offset = (input_idx as f32 * 1.618).rem_euclid(2.0 * PI);
        
        // Decay factor (Improvement 4: M = (1-lambda)M + alpha*Update)
        let lambda = 0.002;
        let dim_norm = (self.dim as f64).sqrt();

        for i in 0..self.dim {
            let next_i = (i + 1) % self.dim;

            // 1. Pointwise correlation (Standard PP-CEL)
            let psi_re = self.psi_real[i] as f64;
            let psi_im = self.psi_imag[i] as f64;
            let psi_mag = (psi_re.powi(2) + psi_im.powi(2)).sqrt() + 1e-9;
            let u_psi_re = psi_re / psi_mag;
            let u_psi_im = psi_im / psi_mag;

            let sig_phase = self.scramble_phases[i] + offset;
            let (sig_sin, sig_cos) = sig_phase.sin_cos();
            let sig_re = sig_cos as f64;
            let sig_im = sig_sin as f64;

            let corr_re = u_psi_re * sig_re + u_psi_im * sig_im;
            let corr_im = u_psi_im * sig_re - u_psi_re * sig_im;
            
            self.q_memory_re[i] = self.q_memory_re[i] * (1.0 - lambda) + corr_re * alpha / dim_norm;
            self.q_memory_im[i] = self.q_memory_im[i] * (1.0 - lambda) + corr_im * alpha / dim_norm;

            // 2. Topological Gradient Correlation (Phase differences)
            let psi_re_next = self.psi_real[next_i] as f64;
            let psi_im_next = self.psi_imag[next_i] as f64;
            let psi_mag_next = (psi_re_next.powi(2) + psi_im_next.powi(2)).sqrt() + 1e-9;
            let u_psi_re_next = psi_re_next / psi_mag_next;
            let u_psi_im_next = psi_im_next / psi_mag_next;

            // Delta PSI (Relative phase between neighbors)
            let d_psi_re = u_psi_re * u_psi_re_next + u_psi_im * u_psi_im_next;
            let d_psi_im = u_psi_im * u_psi_re_next - u_psi_re * u_psi_im_next;

            let sig_phase_next = self.scramble_phases[next_i] + offset;
            let (sig_sin_next, sig_cos_next) = sig_phase_next.sin_cos();
            
            // Delta SIG (Relative phase of input signature)
            let d_sig_re = (sig_cos * sig_cos_next as f32 + sig_sin * sig_sin_next as f32) as f64;
            let d_sig_im = (sig_sin * sig_cos_next as f32 - sig_cos * sig_sin_next as f32) as f64;

            // Correlate the "shapes" (phase twists)
            let topo_re = d_psi_re * d_sig_re + d_psi_im * d_sig_im;
            let topo_im = d_psi_im * d_sig_re - d_psi_re * d_sig_im;

            self.q_topo_re[i] = self.q_topo_re[i] * (1.0 - lambda) + topo_re * alpha / dim_norm;
            self.q_topo_im[i] = self.q_topo_im[i] * (1.0 - lambda) + topo_im * alpha / dim_norm;
        }

        // Keep memories bounded
        self.normalize_q_memory(5.0);
        self.normalize_q_topo(3.0);
    }

    fn normalize_q_topo(&mut self, target_norm: f64) {
        let mut total_energy_sq = 0.0;
        for i in 0..self.dim { total_energy_sq += self.q_topo_re[i].powi(2) + self.q_topo_im[i].powi(2); }
        let norm = total_energy_sq.sqrt();
        if norm > 1e-12 {
            let factor = target_norm / norm;
            for i in 0..self.dim { self.q_topo_re[i] *= factor; self.q_topo_im[i] *= factor; }
        }
    }

    fn normalize_q_memory(&mut self, target_norm: f64) {
        let mut total_energy_sq = 0.0;
        for i in 0..self.dim { total_energy_sq += self.q_memory_re[i].powi(2) + self.q_memory_im[i].powi(2); }
        let norm = total_energy_sq.sqrt();
        if norm > 1e-12 {
            let factor = target_norm / norm;
            for i in 0..self.dim { self.q_memory_re[i] *= factor; self.q_memory_im[i] *= factor; }
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
        let dim_scale = (self.dim as f32).sqrt();

        // --- 1. PP-CEL Retrieval (Phase-Gated Key Matching) ---
        // Retrieve a superposed "recall wave" by gating the correlation between 
        // current input_signature and q_memory using Cosine Similarity.
        let mut recall_re = vec![0.0; self.dim];
        let mut recall_im = vec![0.0; self.dim];
        
        // Gate threshold (Improvement 2: theta)
        // High temp = lower threshold (allow more noise), Low temp = stricter matching.
        let gate_theta = (0.3 + system_temp * 0.4).clamp(0.2, 0.8);

        for i in 0..self.dim {
            let next_i = (i + 1) % self.dim;
            let sig_re = self.input_signature[i] as f64;
            let sig_im = (self.scramble_phases[i] + self.rng_seed as f32).sin() as f64 * 0.1;

            let sig_mag = (sig_re.powi(2) + sig_im.powi(2)).sqrt() + 1e-9;
            let u_sig_re = sig_re / sig_mag;
            let u_sig_im = sig_im / sig_mag;

            // 1. Pointwise Recall
            let rec_re = self.q_memory_re[i] * u_sig_re - self.q_memory_im[i] * u_sig_im;
            let rec_im = self.q_memory_re[i] * u_sig_im + self.q_memory_im[i] * u_sig_re;

            // 2. Topological Shape Matching (Coherence Filter)
            // Calculate current local gradient in the query
            let sig_re_next = self.input_signature[next_i] as f64;
            let sig_im_next = (self.scramble_phases[next_i] + self.rng_seed as f32).sin() as f64 * 0.1;
            let sig_mag_next = (sig_re_next.powi(2) + sig_im_next.powi(2)).sqrt() + 1e-9;
            
            // Delta SIG (Relative phase of current query neighbors)
            let d_sig_re = (u_sig_re * (sig_re_next / sig_mag_next) + u_sig_im * (sig_im_next / sig_mag_next));
            let d_sig_im = (u_sig_im * (sig_re_next / sig_mag_next) - u_sig_re * (sig_im_next / sig_mag_next));

            // Matching with stored Topological patterns
            let topo_match = (self.q_topo_re[i] * d_sig_re + self.q_topo_im[i] * d_sig_im).max(0.0);
            let shape_coherence = (topo_match as f32 * 2.0).clamp(0.5, 2.5);

            // g(corr): Cosine Similarity Gate with Shape Coherence boost
            let corr_strength = (rec_re.powi(2) + rec_im.powi(2)).sqrt();
            let gate = if corr_strength > (gate_theta as f64 / shape_coherence as f64) {
                ((corr_strength * shape_coherence as f64) - gate_theta as f64).max(0.0) / (1.0 - gate_theta as f64)
            } else {
                0.0
            };

            recall_re[i] = (rec_re * gate) as f32;
            recall_im[i] = (rec_im * gate) as f32;
        }

        // --- 2. Dynamic Energy Landscape (V) with Thermal Fluctuation ---
        // High temp = high fluctuation + more smoothing (flatness)
        let thermal_noise = (system_temp * 0.3).max(0.01);
        let smoothing = (system_temp * 0.4).clamp(0.1, 0.95);

        for i in 0..self.dim {
            let recall_intensity = (recall_re[i].powi(2) + recall_im[i].powi(2)).sqrt();
            let penalty = penalty_field.get(i).cloned().unwrap_or(0.0);
            
            // Stochastic V: Base + Noise
            let noise = (self.next_rng() - 0.5) * thermal_noise;
            let target_v = -recall_intensity * 2.0 * focus_factor + penalty * 5.0 + noise;
            
            // Landscape smoothing prevents rapid local pinning
            self.energy_landscape[i] = self.energy_landscape[i] * smoothing + target_v * (1.0 - smoothing);
        }

        // --- 3. Wave Evolution ---
        for i in 0..self.dim {
            self.theta[i] *= solidification;
            self.theta[i + self.dim] *= solidification;

            let (re, im) = (self.psi_real[i], self.psi_imag[i]);
            
            // Energy-driven phase shift: omega_eff = omega + V
            let v = self.energy_landscape[i];
            let phase_shift = (self.frequencies[i] + v) * effective_dt;
            let (sin_w, cos_w) = phase_shift.sin_cos();   
            
            let mut new_re = re * cos_w - im * sin_w;
            let mut new_im = re * sin_w + im * cos_w;

            // Direct injection of recalled statistical patterns (Tunneling/Superposition)
            let recall_boost = (1.0 + focus_factor) * (1.0 / (system_temp + 0.1));
            new_re += recall_re[i] * recall_boost * effective_dt;
            new_im += recall_im[i] * recall_boost * effective_dt;

            // Spatial coupling (Resonance)
            let neighbor_re = self.psi_real[(i + 1) % self.dim] + self.psi_real[if i == 0 { self.dim - 1 } else { i - 1 }];
            let coupling = self.theta[i] * neighbor_re / dim_scale;
            
            self.psi_real[i] = new_re + coupling * effective_dt;
            self.psi_imag[i] = new_im;

            // Viscosity / Damping
            let penalty = penalty_field.get(i).cloned().unwrap_or(0.0);
            let viscosity = 0.02 * (1.0 + penalty);
            self.psi_real[i] *= (1.0 - viscosity * effective_dt).max(0.0);
            self.psi_imag[i] *= (1.0 - viscosity * effective_dt).max(0.0);
        }

        // Gravity field (now derived from recall and psi coincidence)
        for i in 0..self.dim {
            let coincidence = (self.psi_real[i] * recall_re[i] + self.psi_imag[i] * recall_im[i]).max(0.0);
            self.gravity_field[i] = self.gravity_field[i] * 0.98 + coincidence * 0.02;
        }

        // --- 4. Boltzmann-like Multimodal Gating ---
        // Allow multiple solution peaks to coexist based on temperature.
        let mut total_e = 0.0;
        for i in 0..self.dim { total_e += self.psi_real[i].powi(2) + self.psi_imag[i].powi(2); }
        let avg_e = total_e / self.dim as f32;
        
        // Beta: Inverse temperature. High temp = low beta = uniform gating.
        let beta = (1.5 / (system_temp + 0.5)).clamp(0.5, 3.0);

        for i in 0..self.dim {
            let e = self.psi_real[i].powi(2) + self.psi_imag[i].powi(2);
            let ratio = e / (avg_e + 1e-6);
            
            // Soft gating: allow multiple peaks that are above avg_e.
            let gate = ratio.powf(beta).clamp(0.1, 4.0);
            self.psi_real[i] *= gate;
            self.psi_imag[i] *= gate;
        }

        let target_norm = 1.0 + (system_temp * 0.5).min(1.5);
        self.normalize(target_norm);
    }

    /// Sets the current input query signature for Q-CEL retrieval.
    /// Distributed signature for better multimodal overlap.
    pub fn set_input_query(&mut self, input_idx: usize, strength: f32) {
        let offset = (input_idx as f32 * 1.618).rem_euclid(2.0 * PI);
        let spread = 3; // Number of neighboring indices to influence

        for i in 0..self.dim {
            self.input_signature[i] *= 0.7; // Momentum-like decay
        }

        for j in 0..spread {
            let idx_offset = (offset + j as f32 * 0.1).rem_euclid(2.0 * PI);
            let weight = 1.0 / (j + 1) as f32;
            for i in 0..self.dim {
                let sig_phase = self.scramble_phases[i] + idx_offset;
                self.input_signature[i] += sig_phase.cos() * strength * weight;
            }
        }

        // Improvement 3: Normalize signature energy
        let mut total_sig = 1e-9;
        for i in 0..self.dim { total_sig += self.input_signature[i].powi(2); }
        let sig_norm = total_sig.sqrt();
        for i in 0..self.dim { self.input_signature[i] /= sig_norm; }
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

    pub fn get_action_scores(&mut self, offset: usize, size: usize, exploration_noise: f32, penalty_field: &[f32]) -> Vec<f32> {
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
            
            // Scaled Score Normalization (similar to Transformer's 1/sqrt(d))
            // Prevents score explosion as the number of bins increases.
            score /= (bin_per_action as f32).sqrt();
            
            // Linear score. Noise/Jitter is now handled at the decision level (Top-k Softmax).
            scores.push(score);
        }
        scores
    }

    pub fn adapt(&mut self, state_idx: usize, reward: f32, last_actions: &[usize], system_temp: f32, action_size: usize) {
        // 高次元ほど学習を慎重に（勾配爆発的な位相変化を防ぐ）
        let dim_factor = (1024.0 / self.dim as f32).sqrt().min(1.0);
        let annealing = (system_temp * 0.5).clamp(0.1, 1.0);
        let base_lr = 1.2 * annealing * dim_factor; 
        let bin_per_action = self.dim / action_size;
        let t_len = self.theta.len();

        for &action_idx in last_actions {
            let base_idx = action_idx * bin_per_action;

            if reward > 1.2 {
                // 強力な報酬：重力場を形成（ブラックホール化）
                for j in 0..bin_per_action {
                    let idx = (base_idx + j) % self.dim;
                    self.gravity_field[idx] = (self.gravity_field[idx] + 0.1 * dim_factor).min(1.0);
                }
                
                // --- Q-CEL Imprinting ---
                // Confident patterns (low temp) are imprinted with higher fidelity.
                let fidelity = (1.1 - system_temp * 0.5).clamp(0.2, 1.0);
                self.imprint_qcel(state_idx, reward * fidelity as f32);
            }

            if reward < 0.0 {
                for j in 0..bin_per_action {
                    let idx = (base_idx + j) % self.dim;
                    self.frequencies[idx] = (self.frequencies[idx] + 0.001).clamp(0.0, 2.0 * PI);
                    self.gravity_field[idx] *= 0.8; // 失敗は重力を弱める
                }
            }
            for neighborhood in -1..=1 {
                let weight = if neighborhood == 0 { 1.0 } else { 0.1 }; // Restore to 0.1
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
                        self.psi_real[idx] += 3.0 * reward * cos_p * dim_factor;
                        self.psi_imag[idx] += 3.0 * reward * sin_p * dim_factor;
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

    /// 負のフィードバックに基づき、行動を抑制するための逆方向アライメント
    pub fn suppress_action(&mut self, action_idx: usize, strength: f32, action_size: usize) {
        let bin_per_action = self.dim / action_size;
        let base_idx = (action_idx * bin_per_action) % self.dim;
        let lr = 0.5 * strength;

        for j in 0..bin_per_action {
            let idx = (base_idx + j) % self.dim;
            let current_phase = self.psi_imag[idx].atan2(self.psi_real[idx]);
            // 逆位相である PI をターゲットにする
            let target_phase = PI;
            let phase_diff_sin = (target_phase - current_phase).sin();
            self.theta[idx] = (self.theta[idx] + phase_diff_sin * lr).clamp(-PI, PI);
            
            // 波動の振幅を減衰させる
            self.psi_real[idx] *= 1.0 - (0.1 * strength);
            self.psi_imag[idx] *= 1.0 - (0.1 * strength);

            // 重力場を弱める
            self.gravity_field[idx] = (self.gravity_field[idx] - 0.02 * strength).max(0.0);
        }
    }

    pub fn inject_exploration_noise(&mut self, strength: f32) {
        for i in 0..self.dim {
            let noise = (self.next_rng() - 0.5) * 2.0;
            self.psi_real[i] += noise * strength;
        }
    }

    /// 特定のアクション領域（Bin）にエネルギーを集中照射し、探索を促す
    pub fn illuminate_bin(&mut self, action_idx: usize, action_size: usize, strength: f32) {
        let bin_per_action = self.dim / action_size;
        let start_idx = (action_idx * bin_per_action) % self.dim;
        
        for i in 0..bin_per_action {
            let idx = (start_idx + i) % self.dim;
            let noise = (self.next_rng() - 0.5) * 0.2;
            // 位相をある程度揃えて注入することで、ノイズよりも強い「指向性」を持たせる
            self.psi_real[idx] += (1.0 + noise) * strength;
            self.psi_imag[idx] += noise * strength;
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

    pub fn calculate_ipr(&self) -> f32 {
        let mut ipr = 0.0;
        let mut norm_sq = 0.0;
        for i in 0..self.dim {
            let e = self.psi_real[i].powi(2) + self.psi_imag[i].powi(2);
            ipr += e * e;
            norm_sq += e;
        }
        // 正規化してD不変にする
        if norm_sq > 1e-10 {
            ipr / (norm_sq * norm_sq) * self.dim as f32
        } else {
            0.0
        }
    }
}

/// 複数の1024次元MWSOシャードの直和空間
/// H_total = H_0 ⊕ H_1 ⊕ ... ⊕ H_n
/// 計算量O(1024)×シャード数、表現能力はシャード数×1024
pub struct ShardedMWSO {
    pub shards: Vec<MWSO>,
    pub shard_dim: usize,       // 各シャードの次元（固定1024）
    pub total_action_size: usize,
    pub actions_per_shard: usize,
    // (from_shard, from_bin, to_shard, to_bin) -> strength
    pub inter_shard_tunnels: HashMap<(usize, usize, usize, usize), f32>,
    // 状態とシャードの親和性 (state_idx -> shard_affinities)
    pub state_affinities: HashMap<usize, Vec<f32>>,
}

impl ShardedMWSO {
    pub fn new(total_action_size: usize) -> Self {
        let shard_dim = 1024;
        // 1シャードあたり何アクションを担当するか
        // shard_dim / 64 = 16アクション/シャード（N=64D則を維持）
        let actions_per_shard = shard_dim / 64;
        let num_shards = (total_action_size + actions_per_shard - 1) / actions_per_shard;
 
        let shards = (0..num_shards).map(|_| MWSO::new(shard_dim)).collect();
 
        Self {
            shards,
            shard_dim,
            total_action_size,
            actions_per_shard,
            inter_shard_tunnels: HashMap::new(),
            state_affinities: HashMap::new(),
        }
    }
 
    /// 新設：シャード間の情報伝達トンネルを追加・強化する
    pub fn add_or_strengthen_tunnel(&mut self, from_shard_idx: usize, to_shard_idx: usize, from_state_idx: usize, local_to_action: usize, strength: f32) {
        if from_shard_idx >= self.shards.len() || to_shard_idx >= self.shards.len() || from_shard_idx == to_shard_idx {
            return;
        }

        // 状態インデックスを、担当シャード内の代表ビンに変換
        let from_bin = from_state_idx % self.shard_dim;

        // アクションインデックスを、担当シャード内の代表ビンに変換
        let bin_per_action = self.shard_dim / self.actions_per_shard;
        let to_bin = local_to_action * bin_per_action;

        let key = (from_shard_idx, from_bin, to_shard_idx, to_bin);
        let tunnel_strength = self.inter_shard_tunnels.entry(key).or_insert(0.0);
        *tunnel_strength = (*tunnel_strength + strength).min(1.0);
    }

    /// どのシャードが担当するかを返す
    pub fn shard_for_action(&self, action_idx: usize) -> (usize, usize) {
        let shard_idx = action_idx / self.actions_per_shard;
        let local_action = action_idx % self.actions_per_shard;
        (shard_idx.min(self.shards.len() - 1), local_action)
    }

    pub fn get_action_scores(&mut self, penalty_field: &[f32]) -> Vec<f32> {
        let mut scores = Vec::with_capacity(self.total_action_size);
        let bin_per_action = self.shard_dim / self.actions_per_shard;

        for shard_idx in 0..self.shards.len() {
            let action_start = shard_idx * self.actions_per_shard;
            let action_end = (action_start + self.actions_per_shard).min(self.total_action_size);
            let local_size = action_end - action_start;

            let slice_start = action_start * bin_per_action;
            let slice_end = action_end * bin_per_action;
            
            let mut local_penalty = vec![0.0f32; self.shard_dim];
            if slice_end > slice_start && slice_end <= penalty_field.len() {
                 let relevant_slice = &penalty_field[slice_start..slice_end];
                 let local_penalty_len = relevant_slice.len();
                 local_penalty[..local_penalty_len].copy_from_slice(relevant_slice);
            }

            let shard_scores = self.shards[shard_idx]
                .get_action_scores(0, local_size, 0.0, &local_penalty);
            scores.extend_from_slice(&shard_scores);
        }
        scores
    }
 
    pub fn inject_state(&mut self, state_idx: usize, strength: f32, system_temp: f32, penalty_field: &[f32]) {
        if self.shards.is_empty() { return; }

        let num_shards = self.shards.len();
        
        // 1. 状態に対するシャード親和性を取得・初期化
        let affinities = self.state_affinities.entry(state_idx).or_insert_with(|| {
            // 最初は全シャードに等しく分配（探索のため）
            vec![1.0; num_shards]
        });

        // 2. 温度に応じた探索・分配重みの計算
        // 高温時は全シャードに均等に近く、低温時は親和性の高いシャードに集中させる
        let mut weights = vec![0.0; num_shards];
        let mut total_weight = 0.0;
        
        // 温度が高いほど(2.0に近い)、base_prob が高くなり、全シャードが均等に選ばれやすくなる
        let exploration_factor = (system_temp * 0.5).clamp(0.0, 1.0);
        let base_prob = exploration_factor * (1.0 / num_shards as f32);

        for i in 0..num_shards {
            // 親和性と探索因子のハイブリッド重み
            weights[i] = (affinities[i] * (1.0 - exploration_factor)) + base_prob;
            total_weight += weights[i];
        }

        // 重みの正規化
        if total_weight > 0.0 {
            for w in &mut weights { *w /= total_weight; }
        }

        // 3. 各シャードに重みに応じた強度で注入
        let bin_per_action = self.shard_dim / self.actions_per_shard;
        
        // 動的なノイズカット閾値（シャード数に応じた平均の半分以下なら切る）
        let cutoff_threshold = (1.0 / num_shards as f32) * 0.5 * (1.0 - system_temp).max(0.1);

        for shard_idx in 0..num_shards {
            let shard_weight = weights[shard_idx];
            
            // 閾値未満のシャードには状態を注入しない（無駄な波を立てない）
            if shard_weight < cutoff_threshold { continue; }

            let action_start = shard_idx * self.actions_per_shard;
            let action_end = (action_start + self.actions_per_shard).min(self.total_action_size);

            let slice_start = action_start * bin_per_action;
            let slice_end = action_end * bin_per_action;

            let mut local_penalty = vec![0.0f32; self.shard_dim];
            if slice_end > slice_start && slice_end <= penalty_field.len() {
                let relevant_slice = &penalty_field[slice_start..slice_end];
                let local_penalty_len = relevant_slice.len();
                local_penalty[..local_penalty_len].copy_from_slice(relevant_slice);
            }

            let shard = &mut self.shards[shard_idx];
            // Q-CEL: Set the input query for this shard
            shard.set_input_query(state_idx, strength * shard_weight);
            // Also inject some energy into the state representation
            shard.inject_state(state_idx % shard.dim, strength * shard_weight, &local_penalty);
        }
    }
 
    pub fn step_core(&mut self, dt: f32, speed_boost: f32, focus_factor: f32, system_temp: f32, penalty_field: &[f32]) {
        let bin_per_action = self.shard_dim / self.actions_per_shard;

        // 1. 各シャードを独立して時間発展させる
        for (shard_idx, shard) in &mut self.shards.iter_mut().enumerate() {
            let action_start = shard_idx * self.actions_per_shard;
            let action_end = (action_start + self.actions_per_shard).min(self.total_action_size);

            let slice_start = action_start * bin_per_action;
            let slice_end = action_end * bin_per_action;
            
            let mut local_penalty = vec![0.0f32; self.shard_dim];
            if slice_end > slice_start && slice_end <= penalty_field.len() {
                 let relevant_slice = &penalty_field[slice_start..slice_end];
                 let local_penalty_len = relevant_slice.len();
                 local_penalty[..local_penalty_len].copy_from_slice(relevant_slice);
            }

            shard.step_core(dt, speed_boost, focus_factor, system_temp, &local_penalty);
        }

        // 2. シャード間トンネルでエネルギーを交換する
        let effective_dt = dt * (1.0 + speed_boost);
        for (&(from_shard, from_bin, to_shard, to_bin), &strength) in &self.inter_shard_tunnels {
            // エネルギーを失う側（from）
            let (from_psi_re, from_psi_im) = {
                let shard_from = &self.shards[from_shard];
                (shard_from.psi_real[from_bin], shard_from.psi_imag[from_bin])
            };
            
            let delta_re = from_psi_re * strength * effective_dt;
            let delta_im = from_psi_im * strength * effective_dt;

            // エネルギーを得る側（to）
            let shard_to = &mut self.shards[to_shard];
            shard_to.psi_real[to_bin] += delta_re;
            shard_to.psi_imag[to_bin] += delta_im;
        }

        // 3. トンネル自体の自然減衰と枝刈り
        self.inter_shard_tunnels.retain(|_, strength| {
            *strength *= 0.995;
            *strength > 0.01
        });
    }
 
    pub fn adapt(&mut self, state_idx: usize, reward: f32, last_actions: &[usize], system_temp: f32) {
        for &action_idx in last_actions {
            let (shard_idx, local_action) = self.shard_for_action(action_idx);
            self.shards[shard_idx].adapt(
                state_idx,
                reward,
                &[local_action],
                system_temp,
                self.actions_per_shard,
            );

            // 成功した場合、そのシャードと状態の親和性を強化
            if reward > 0.1 {
                let affinities = self.state_affinities.entry(state_idx).or_insert_with(|| {
                    vec![1.0 / self.shards.len() as f32; self.shards.len()]
                });
                
                // 親和性の強化
                affinities[shard_idx] = (affinities[shard_idx] + reward * 0.1).min(2.0);

                // 他のシャードの親和性を相対的に減衰（競合）
                for i in 0..affinities.len() {
                    if i != shard_idx {
                        affinities[i] *= 0.95;
                    }
                }
            }
        }
    }
 
    pub fn calculate_ipr(&self) -> f32 {
        // 全シャードのIPR平均
        self.shards.iter().map(|s| s.calculate_ipr()).sum::<f32>() / self.shards.len() as f32
    }
 
    pub fn calculate_rhyd(&self) -> f32 {
        self.shards.iter().map(|s| s.calculate_rhyd()).sum::<f32>() / self.shards.len() as f32
    }
 
    pub fn num_shards(&self) -> usize {
        self.shards.len()
    }

    pub fn illuminate_bin(&mut self, action_idx: usize, strength: f32) {
        let (shard_idx, local_action) = self.shard_for_action(action_idx);
        self.shards[shard_idx].illuminate_bin(
            local_action,
            self.actions_per_shard,
            strength,
        );
    }

    pub fn align_to_action(&mut self, action_idx: usize, strength: f32) {
        let (shard_idx, local_action) = self.shard_for_action(action_idx);
        self.shards[shard_idx].align_to_action(
            local_action,
            strength,
            self.actions_per_shard,
        );
    }

    pub fn suppress_action(&mut self, action_idx: usize, strength: f32) {
        let (shard_idx, local_action) = self.shard_for_action(action_idx);
        self.shards[shard_idx].suppress_action(
            local_action,
            strength,
            self.actions_per_shard,
        );
    }

    pub fn inject_state_for_action(&mut self, state_idx: usize, action_idx: usize, strength: f32, penalty_field: &[f32]) {
        let (shard_idx, _) = self.shard_for_action(action_idx);
        
        let bin_per_action = self.shard_dim / self.actions_per_shard;
        let action_start = shard_idx * self.actions_per_shard;
        let action_end = (action_start + self.actions_per_shard).min(self.total_action_size);

        let slice_start = action_start * bin_per_action;
        let slice_end = action_end * bin_per_action;

        let mut local_penalty = vec![0.0f32; self.shard_dim];
        if slice_end > slice_start && slice_end <= penalty_field.len() {
             let relevant_slice = &penalty_field[slice_start..slice_end];
             let local_penalty_len = relevant_slice.len();
             local_penalty[..local_penalty_len].copy_from_slice(relevant_slice);
        }

        self.shards[shard_idx].inject_state(
            state_idx % self.shard_dim, strength, &local_penalty
        );
    }
}
