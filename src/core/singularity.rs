use super::horizon::Horizon;
use super::node::Node;
use super::mwso::MWSO;
use std::fs::File;
use std::io::{self, Read, Write};

pub struct Singularity {
    pub nodes: Vec<Node>,
    pub horizon: Horizon,
    pub mwso: MWSO,
    pub bootstrapper: crate::core::knowledge::Bootstrapper,
    pub active_conditions: Vec<i32>, 
    pub system_temperature: f32,
    pub last_topology_update_temp: f32,
    pub adrenaline: f32,
    pub frustration: f32,
    pub velocity_trust: f32,
    pub fatigue_map: Vec<f32>,
    pub morale: f32,
    pub patience: f32,
    pub category_sizes: Vec<usize>, 
    pub action_size: usize,    
    pub state_size: usize,
    pub last_actions: Vec<usize>, 
    pub last_state_idx: usize,
    pub learned_rules: Vec<(usize, usize, usize)>, 
    pub penalty_matrix: Vec<f32>, // 状態×次元の広大なペナルティ空間

    pub exploration_beta: f32,    

    pub idx_aggression: usize,
    pub idx_fear: usize,
    pub idx_tactical: usize,
    pub idx_reflex: usize,
}

impl Singularity {
    pub fn new(state_size: usize, category_sizes: Vec<usize>) -> Self {
        let nodes = vec![Node::new(0.5), Node::new(0.4), Node::new(0.3), Node::new(0.3)];
        let total_action_size: usize = category_sizes.iter().sum();
        let required_dim = (total_action_size * 64).next_power_of_two().max(512);
        
        Self {
            nodes,
            horizon: Horizon::new(),
            mwso: MWSO::new(required_dim),
            bootstrapper: crate::core::knowledge::Bootstrapper::new(),
            active_conditions: Vec::new(),
            system_temperature: 0.5,
            last_topology_update_temp: -1.0,
            adrenaline: 0.0,
            frustration: 0.0,
            velocity_trust: 1.0,
            fatigue_map: vec![0.0; total_action_size],
            morale: 1.0,
            patience: 1.0,
            category_sizes: category_sizes.clone(),
            action_size: total_action_size,
            state_size,
            last_actions: vec![0; category_sizes.len()],
            last_state_idx: 0,
            learned_rules: Vec::new(),
            penalty_matrix: vec![0.0; state_size * required_dim],
            exploration_beta: 0.1, 
            idx_aggression: 0,
            idx_fear: 1,
            idx_tactical: 2,
            idx_reflex: 3,
        }
    }

    pub fn set_active_conditions(&mut self, conditions: &[i32]) {
        self.active_conditions = conditions.to_vec();
    }

    pub fn select_actions(&mut self, state_idx: usize) -> Vec<i32> {
        self.last_state_idx = state_idx;
        let speed_boost = (self.adrenaline * 0.5).clamp(0.0, 1.0);
        let focus_factor = (self.nodes[self.idx_tactical].state * 0.5).clamp(0.0, 1.0);

        // 弾性的失敗記憶の取得
        let start = state_idx * self.mwso.dim;
        // penalty_field を Vec にコピーして借用問題を回避（多少のコストは許容）
        let current_penalty_field = self.penalty_matrix[start..start + self.mwso.dim].to_vec();

        self.mwso.inject_state(state_idx, 1.0, &current_penalty_field);

        if self.active_conditions.is_empty() {
            let noise_strength = (self.system_temperature * 0.1).clamp(0.05, 0.3);
            self.mwso.inject_exploration_noise(noise_strength);
        }
        
        self.mwso.step_core(0.1, speed_boost, focus_factor, self.system_temperature);

        let mut results = Vec::with_capacity(self.category_sizes.len());
        let mut current_offset = 0;
        let cat_sizes = self.category_sizes.clone();

        for (cat_idx, &size) in cat_sizes.iter().enumerate() {
            let best_idx = self.get_best_in_range(current_offset, size, &current_penalty_field);
            self.last_actions[cat_idx] = current_offset + best_idx;
            results.push(best_idx as i32);
            current_offset += size;
        }
        results
    }
// ... (generate_visual_snapshot の復元)
    pub fn generate_visual_snapshot(&self, path: &str) -> bool {
        super::visualizer::Visualizer::render_wave_snapshot(&self.mwso, path).is_ok()
    }

    fn get_best_in_range(&mut self, offset: usize, size: usize, penalty_field: &[f32]) -> usize {
        let noise = if self.active_conditions.is_empty() { 0.2 } else { 0.0 };
        let mwso_scores = self.mwso.get_action_scores(offset, size, noise, penalty_field);
        
        let mut best = 0;
        let mut max_score = -f32::INFINITY;

        for i in 0..size {
            let base_score = mwso_scores[i] - self.fatigue_map[offset + i] * 0.5;
            let internal_field = self.learned_rules.iter()
                .find(|r| r.0 == self.last_state_idx && r.1 == offset + i)
                .map(|r| (r.2 as f32 * 2.0).min(5.0)).unwrap_or(0.0);

            // ハミルトニアンの理性スコア
            let resonance_field = self.bootstrapper.calculate_resonance_field(&self.active_conditions, self.action_size);
            let knowledge_field = resonance_field[offset + i].map(|s| s * 2.5).unwrap_or(0.0);

            let neuron_boost = match i {
                0 => self.nodes[self.idx_aggression].state * 0.4,
                1 => self.nodes[self.idx_fear].state * 0.2,
                _ => 0.0,
            };
            
            let total_score = base_score + internal_field + knowledge_field + neuron_boost + (self.morale * 0.1);
            
            // 尖らせ処理（不確定性維持版）
            let sharp_factor = (10.0 - self.system_temperature * 4.0).clamp(1.0, 10.0);
            let collapsed_score = (total_score + 10.0).max(0.1).powf(sharp_factor) + (i as f32 * 0.01).sin() * 0.001;

            if collapsed_score > max_score {
                max_score = collapsed_score;
                best = i;
            }
        }
        best
    }

    pub fn learn(&mut self, reward: f32) {
        self.mwso.adapt(reward, &self.last_actions, self.system_temperature, self.action_size);

        if self.active_conditions.is_empty() {
            let state = self.last_state_idx;
            let action = self.last_actions[0];
            let bin_per_action = self.mwso.dim / self.action_size;

            if reward > 1.5 {
                if let Some(rule) = self.learned_rules.iter_mut().find(|r| r.0 == state && r.1 == action) {
                    rule.2 += 1;
                } else {
                    self.learned_rules.push((state, action, 1));
                }
                // 正解時はペナルティを急速に減衰
                let start = state * self.mwso.dim + action * bin_per_action;
                for j in 0..bin_per_action { self.penalty_matrix[start + j] *= 0.5; }
            } else if reward < 0.0 {
                // 失敗時はペナルティを蓄積
                let start = state * self.mwso.dim + action * bin_per_action;
                for j in 0..bin_per_action { 
                    self.penalty_matrix[start + j] = (self.penalty_matrix[start + j] + reward.abs() * 2.0).min(10.0); 
                }
            }
        }

        // ペナルティの自然減衰（弾性）
        for p in &mut self.penalty_matrix { *p *= 0.995; }

        for &idx in &self.last_actions {
            if reward < 0.0 { self.fatigue_map[idx] = (self.fatigue_map[idx] + 0.2).min(1.0); }
            else { self.fatigue_map[idx] = (self.fatigue_map[idx] - 0.3).max(0.0); }
        }
        for f in &mut self.fatigue_map { *f *= 0.98; }
        self.digest_experience(reward.abs(), reward, if reward < 0.0 { reward.abs() } else { 0.0 });
    }

    pub fn digest_experience(&mut self, td_error: f32, reward: f32, penalty: f32) {
        if reward > 1.5 { self.system_temperature = 0.05; }
        else if reward > 0.0 {
            let cooling = if self.active_conditions.is_empty() { 0.8 } else { 0.85 };
            self.system_temperature = (self.system_temperature * cooling - reward * 0.1).max(0.05);
        } else {
            self.system_temperature = (self.system_temperature + td_error * 0.2).min(2.0);
        }

        let urgency = ((reward + penalty) * 5.0).min(1.0);
        self.mwso.inject_state(0, reward, &vec![0.0; self.mwso.dim]);
        self.mwso.inject_state(1, -penalty, &vec![0.0; self.mwso.dim]);
        self.mwso.step_core(0.05, 0.0, 0.0, self.system_temperature);

        let current_states: Vec<f32> = self.nodes.iter().map(|n| n.state).collect();
        for node in &mut self.nodes { node.update(0.0, urgency, self.system_temperature, &current_states); }

        if urgency > 0.5 || (self.system_temperature - self.last_topology_update_temp).abs() > 0.05 {
            self.reshape_topology();
        }
        let all_indices: Vec<usize> = (0..self.nodes.len()).collect();
        self.horizon.regulate(self.system_temperature, &all_indices, &mut self.nodes);
    }

    pub fn reshape_topology(&mut self) {
        self.last_topology_update_temp = self.system_temperature;
        let arousal = (self.nodes[self.idx_aggression].state + self.adrenaline).clamp(0.0, 2.0);
        let tactical_focus = self.nodes[self.idx_tactical].state;
        let temp = self.system_temperature;

        self.update_connection(self.idx_tactical, self.idx_reflex, (1.0 - temp).clamp(0.0, 1.0) * (1.0 + tactical_focus));
        self.update_connection(self.idx_aggression, self.idx_reflex, arousal * 1.5);
        self.update_connection(self.idx_fear, self.idx_reflex, self.nodes[self.idx_fear].state * 2.0);

        if self.system_temperature > 1.5 {
            let glia_intervention = self.horizon.get_intervention_level();
            if glia_intervention > 0.7 {
                 self.nodes[self.idx_aggression].apply_inhibition(0.3);
                 self.nodes[self.idx_fear].apply_inhibition(0.2);
            }
        }
        self.apply_elastic_fatigue();
    }

    fn update_connection(&mut self, from: usize, to: usize, weight: f32) {
        if let Some(node) = self.nodes.get_mut(from) {
            if let Some(synapse) = node.synapses.iter_mut().find(|s| s.target_id == to) { synapse.weight = weight; }
            else { node.synapses.push(super::node::Synapse { target_id: to, weight }); }
        }
    }

    fn apply_elastic_fatigue(&mut self) {
        for (idx, fatigue) in self.fatigue_map.iter().enumerate() {
            if *fatigue > 0.8 {
                for node in &mut self.nodes {
                    for s in &mut node.synapses { if s.target_id == idx { s.weight *= 0.5; } }
                }
            }
        }
    }

    pub fn update_all_nodes(&mut self, input_signals: &[f32], urgency: f32) {
        self.mwso.step_core(0.1, 0.0, 0.0, self.system_temperature);
        let current_states: Vec<f32> = self.nodes.iter().map(|n| n.state).collect();
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let input = input_signals.get(i).cloned().unwrap_or(0.0);
            node.update(input, urgency, self.system_temperature, &current_states);
        }
    }

    pub fn set_neuron_state(&mut self, idx: usize, state: f32) {
        if let Some(node) = self.nodes.get_mut(idx) { node.state = state.clamp(0.0, 1.0); }
    }

    pub fn get_resonance_density(&self) -> f32 { self.mwso.calculate_rhyd() }

    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(b"DSYM")?;
        file.write_all(&12u32.to_le_bytes())?; 
        file.write_all(&(self.state_size as u32).to_le_bytes())?;
        file.write_all(&self.system_temperature.to_le_bytes())?;
        file.write_all(&self.adrenaline.to_le_bytes())?;
        file.write_all(&self.frustration.to_le_bytes())?;
        file.write_all(&self.velocity_trust.to_le_bytes())?;
        file.write_all(&self.morale.to_le_bytes())?;
        file.write_all(&self.patience.to_le_bytes())?;
        file.write_all(&self.exploration_beta.to_le_bytes())?;
        file.write_all(&self.horizon.glutamate_buffer.to_le_bytes())?;
        for f in &self.fatigue_map { file.write_all(&f.to_le_bytes())?; }
        file.write_all(&(self.category_sizes.len() as u32).to_le_bytes())?;
        for &s in &self.category_sizes { file.write_all(&(s as u32).to_le_bytes())?; }
        file.write_all(&(self.nodes.len() as u32).to_le_bytes())?;
        for node in &self.nodes {
            file.write_all(&node.state.to_le_bytes())?;
            file.write_all(&node.base_decay.to_le_bytes())?;
        }
        file.write_all(&(self.learned_rules.len() as u32).to_le_bytes())?;
        for &(s, a, count) in &self.learned_rules {
            file.write_all(&(s as u32).to_le_bytes())?;
            file.write_all(&(a as u32).to_le_bytes())?;
            file.write_all(&(count as u32).to_le_bytes())?;
        }
        file.write_all(&(self.mwso.dim as u32).to_le_bytes())?;
        for &f in &self.mwso.psi_real { file.write_all(&f.to_le_bytes())?; }
        for &f in &self.mwso.psi_imag { file.write_all(&f.to_le_bytes())?; }
        file.write_all(&(self.mwso.theta.len() as u32).to_le_bytes())?;
        for &f in &self.mwso.theta { file.write_all(&f.to_le_bytes())?; }
        Ok(())
    }

    pub fn load_from_file(&mut self, path: &str) -> io::Result<()> {
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let mut cur = 0;
        let read_f32 = |p: &mut usize| -> f32 { let v = f32::from_le_bytes(buf[*p..*p+4].try_into().unwrap()); *p+=4; v };
        if &buf[0..4] != b"DSYM" { return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid")); }
        cur += 8; 
        self.system_temperature = read_f32(&mut cur);
        self.adrenaline = read_f32(&mut cur);
        self.frustration = read_f32(&mut cur);
        self.velocity_trust = read_f32(&mut cur);
        self.morale = read_f32(&mut cur);
        self.patience = read_f32(&mut cur);
        self.last_topology_update_temp = -1.0;
        self.reshape_topology();
        Ok(())
    }
}