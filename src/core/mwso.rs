// Monolithic Wave-State Operator (MWSO)
// 512 parameters, complex latent space, PDE-based intelligence.

use std::f32::consts::PI;

pub struct MWSO {
    /// Wave state: 256 complex numbers (512 floats total)
    /// psi_real[i] and psi_imag[i] represent the state in the complex latent space.
    pub psi_real: Vec<f32>,
    pub psi_imag: Vec<f32>,

    /// 512 parameters defining the "refractive index" and interaction logic.
    pub theta: Vec<f32>,

    /// Time constant spectrum: Each component has a different "natural frequency".
    pub frequencies: Vec<f32>,

    pub dim: usize,
}

impl MWSO {
    pub fn new() -> Self {
        let dim = 256;
        let mut theta = vec![0.0; 512];
        let mut frequencies = vec![0.0; dim];
        
        // Initialize theta with some variance
        for i in 0..512 {
            theta[i] = (i as f32 * 0.1).sin() * 0.1;
        }

        // Initialize frequencies (Spectrum of time constants)
        // 0.0 to 1.0 mapping to slow (strategic) to fast (reflexive) waves.
        for i in 0..dim {
            frequencies[i] = (i as f32 / dim as f32).powi(2) * 2.0 * PI;
        }

        let mut psi_real = vec![0.0; dim];
        psi_real[0] = 1.0; // Ground state (initial energy)

        Self {
            psi_real,
            psi_imag: vec![0.0; dim],
            theta,
            frequencies,
            dim,
        }
    }

    /// The Unified Wave Operator (UWO)
    /// Evolves the wave state based on internal dynamics and external input.
    pub fn step(&mut self, input: &[f32], dt: f32) {
        // 1. External Wave Injection (Input as Shock Waves)
        for (i, &val) in input.iter().enumerate() {
            if i < self.dim {
                self.psi_real[i] += val * self.theta[i % 512];
                self.psi_imag[i] += val * self.theta[(i + 256) % 512];
            }
        }

        // 2. Non-linear Wave Propagation (The PDE approximation)
        // dPsi/dt = i * omega * Psi + Interaction(Psi, Theta)
        for i in 0..self.dim {
            let omega = self.frequencies[i];
            
            // Linear phase rotation (Standing waves)
            let re = self.psi_real[i];
            let im = self.psi_imag[i];
            
            // Rotation by omega * dt
            let cos_w = (omega * dt).cos();
            let sin_w = (omega * dt).sin();
            
            let mut new_re = re * cos_w - im * sin_w;
            let mut new_im = re * sin_w + im * cos_w;

            // 2.2 Holographic Interference (遠隔干渉)
            let coupling_idx = i % 512;
            let coupling_strength = self.theta[coupling_idx];
            
            let next_idx = (i + 1) % self.dim;
            let prev_idx = if i == 0 { self.dim - 1 } else { i - 1 };
            
            // 隣接干渉（近接連続性）
            let local_interaction = coupling_strength * (self.psi_real[next_idx] + self.psi_real[prev_idx]);
            
            // 遠隔干渉（ホログラフィック結合）
            // thetaの後半部分をストライドとして使用。
            let stride = (self.theta[(i + 128) % 512].abs() * 128.0) as usize % self.dim;
            let remote_idx = (i + stride) % self.dim;
            let remote_interaction = self.theta[(i + 384) % 512] * self.psi_real[remote_idx];
            
            new_re += (local_interaction + remote_interaction) * dt;
            
            // 2.3 Self-referential feedback (RNN-like resonance)
            let feedback = self.theta[(i + 256) % 512] * (new_re * new_re + new_im * new_im).sqrt();
            new_re *= 1.0 - feedback * 0.01 * dt;
            new_im *= 1.0 - feedback * 0.01 * dt;

            self.psi_real[i] = new_re;
            self.psi_imag[i] = new_im;
        }

        // 3. Normalization (Energy Conservation)
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

    /// Decodes the wave state into action scores for a given category.
    pub fn get_action_scores(&self, offset: usize, size: usize) -> Vec<f32> {
        let mut scores = Vec::with_capacity(size);
        for i in 0..size {
            // Projection of the high-dimensional wave state onto an action "reflector".
            // Use theta as the basis for projection to make it learnable.
            let mut score = 0.0;
            for j in 0..16 { // Use 16-dim projection per action
                let psi_idx = (offset + i * 16 + j) % self.dim;
                let theta_idx = (offset + i * 16 + j) % 512;
                score += self.psi_real[psi_idx] * self.theta[theta_idx];
            }
            scores.push(score);
        }
        scores
    }

    /// Parameter Adaptation (The "Learning" process)
    /// Adjusts the refractive index (theta) based on reward/interference.
    pub fn adapt(&mut self, reward: f32, last_actions: &[usize]) {
        let lr = 0.01;
        for &action_idx in last_actions {
            for j in 0..16 {
                let theta_idx = (action_idx * 16 + j) % 512;
                let psi_idx = (action_idx * 16 + j) % self.dim;
                
                // Hebbian-like update: strengthen the coupling that was active
                let delta = reward * self.psi_real[psi_idx];
                self.theta[theta_idx] += lr * delta;
                
                // Regularization to prevent divergence
                self.theta[theta_idx] = self.theta[theta_idx].clamp(-2.0, 2.0);
            }
        }
    }
}
