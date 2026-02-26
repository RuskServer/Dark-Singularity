use super::horizon::Horizon;
use super::node::Node;
use super::mwso::MWSO;
use std::fs::File;
use std::io::{self, Read, Write};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Experience {
    pub state_idx: usize,
    pub actions: Vec<usize>,
}

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
    pub action_momentum: Vec<f32>, 
    pub input_history: VecDeque<usize>, // 入力状態の履歴（流れ）
    pub history: VecDeque<Experience>,
    pub max_history: usize,
    pub learned_rules: Vec<(usize, usize, usize)>, 
    pub penalty_matrix: Vec<f32>, 

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
            action_momentum: vec![0.0; total_action_size],
            input_history: VecDeque::with_capacity(8),
            history: VecDeque::with_capacity(32),
            max_history: 15,
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

        let start = state_idx * self.mwso.dim;
        let mut current_penalty_field = self.penalty_matrix[start..start + self.mwso.dim].to_vec();

        // --- Knowledge-based Penalty Injection ---
        let bin_per_action = self.mwso.dim / self.action_size;
        let active_resonance = self.bootstrapper.calculate_resonance_field(&self.active_conditions, self.action_size);
        for (action_idx, strength_opt) in active_resonance.iter().enumerate() {
            if let Some(strength) = strength_opt {
                if *strength < 0.0 {
                    let p_val = strength.abs() * 50.0; // ペナルティ強度を増幅して注入
                    let b_start = action_idx * bin_per_action;
                    for j in 0..bin_per_action {
                        if b_start + j < current_penalty_field.len() {
                            current_penalty_field[b_start + j] += p_val;
                        }
                    }
                }
            }
        }

        // --- Flow Injection (Temporal Smearing) ---
        // 現在の状態を 1.0 で注入
        self.mwso.inject_state(state_idx, 1.0, &current_penalty_field);
        
        // 過去の状態を減衰させながら重畳注入（流れを形成）
        let mut decay = 0.4;
        for &prev_idx in self.input_history.iter().rev() {
            self.mwso.inject_state(prev_idx, decay, &current_penalty_field);
            decay *= 0.5;
            if decay < 0.1 { break; }
        }
        
        // 履歴の更新
        self.input_history.push_back(state_idx);
        if self.input_history.len() > 4 { self.input_history.pop_front(); }
        // ------------------------------------------

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

        self.history.push_back(Experience {
            state_idx,
            actions: self.last_actions.clone(),
        });
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }

        results
    }

    pub fn generate_visual_snapshot(&self, path: &str) -> bool {
        super::visualizer::Visualizer::render_wave_snapshot(&self.mwso, path).is_ok()
    }

    fn get_best_in_range(&mut self, offset: usize, size: usize, penalty_field: &[f32]) -> usize {
        let noise = if self.active_conditions.is_empty() { 0.2 } else { 0.0 };
        let mwso_scores = self.mwso.get_action_scores(offset, size, noise, penalty_field);
        
        let mut best = 0;
        let mut max_score = -f32::INFINITY;

        let active_resonance = self.bootstrapper.calculate_resonance_field(&self.active_conditions, self.action_size);

        for i in 0..size {
            let mut knowledge_field = 0.0;
            if let Some(s) = active_resonance[offset + i] {
                if s < -0.9 { // 強力なペナルティ（無限大に近い排斥）
                    knowledge_field = -1e6; 
                } else {
                    knowledge_field = s * 30.0;
                }
            }
            
            let base_score = mwso_scores[i] - self.fatigue_map[offset + i] * 0.5;
            let internal_field = self.learned_rules.iter()
                .find(|r| r.0 == self.last_state_idx && r.1 == offset + i)
                .map(|r| (r.2 as f32 * 2.0).min(5.0)).unwrap_or(0.0);

            // 現在の入力状態に合致するルールがあれば、それを「動機」として加算
            if let Some(rule) = self.bootstrapper.rules.iter().find(|r| r.condition_id == self.last_state_idx as i32 && r.target_action == offset + i) {
                knowledge_field += rule.strength * 20.0;
            }

            let neuron_boost = match i {
                0 => self.nodes[self.idx_aggression].state * 0.4,
                1 => self.nodes[self.idx_fear].state * 0.2,
                _ => 0.0,
            };
            
            let momentum_boost = self.action_momentum[offset + i] * 1.5;
            
            let total_score = base_score + internal_field + knowledge_field + neuron_boost + momentum_boost + (self.morale * 0.1);
            
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
        let mut discount = 1.0;
        let gamma = 0.9;
        let bin_per_action = self.mwso.dim / self.action_size;

        for exp in self.history.iter().rev() {
            let discounted_reward = reward * discount;
            self.mwso.adapt(discounted_reward, &exp.actions, self.system_temperature, self.action_size);

            if self.active_conditions.is_empty() {
                let state = exp.state_idx;
                let action = exp.actions[0];

                if discounted_reward > 1.2 {
                    if let Some(rule) = self.learned_rules.iter_mut().find(|r| r.0 == state && r.1 == action) {
                        rule.2 += 1;
                    } else {
                        self.learned_rules.push((state, action, 1));
                    }
                    let start = state * self.mwso.dim + action * bin_per_action;
                    for j in 0..bin_per_action { self.penalty_matrix[start + j] *= 0.5; }
                } else if discounted_reward < 0.0 {
                    let start = state * self.mwso.dim + action * bin_per_action;
                    for j in 0..bin_per_action { 
                        self.penalty_matrix[start + j] = (self.penalty_matrix[start + j] + discounted_reward.abs() * 2.0).min(10.0); 
                    }
                }
            }

            for &idx in &exp.actions {
                if discounted_reward < 0.0 { self.fatigue_map[idx] = (self.fatigue_map[idx] + 0.2 * discount).min(1.0); }
                else { self.fatigue_map[idx] = (self.fatigue_map[idx] - 0.3 * discount).max(0.0); }
            }

            discount *= gamma;
            if discount < 0.01 { break; }
        }

        // 慣性（Momentum）の更新
        if reward > 0.1 {
            for &idx in &self.last_actions {
                self.action_momentum[idx] = (self.action_momentum[idx] + 0.2 * reward).min(2.0);
            }
        } else if reward < -0.5 {
            // 強いペナルティ時は慣性を大幅にリセット（即座に方向転換）
            for m in &mut self.action_momentum { *m *= 0.2; }
        }
        
        // 慣性の自然減衰
        for m in &mut self.action_momentum { *m *= 0.95; }

        for p in &mut self.penalty_matrix { *p *= 0.995; }
        for f in &mut self.fatigue_map { *f *= 0.98; }

        self.digest_experience(reward.abs(), reward, if reward < 0.0 { reward.abs() } else { 0.0 });
        self.history.clear();
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

    /// 逆強化学習: 行動から動機を逆算する
    /// エキスパートの行動を観測し、それを引き起こす「ハミルトニアン場（動機）」を内省的に生成する
    pub fn observe_expert(&mut self, state_idx: usize, expert_actions: &[usize], strength: f32) {
        // 1. 位相の同調（模倣位相ロック）
        for &action in expert_actions {
            self.mwso.align_to_action(action, strength, self.action_size);
        }

        // 2. 動機の逆算と定着（ハミルトニアンルールの自動生成）
        if strength > 0.5 {
            for &action in expert_actions {
                // すでに類似のルールがあるか確認し、あれば強化、なければ新設
                if let Some(rule) = self.bootstrapper.rules.iter_mut()
                    .find(|r| r.condition_id == state_idx as i32 && r.target_action == action) {
                    rule.strength = (rule.strength + 0.1 * strength).min(10.0);
                } else {
                    self.bootstrapper.add_hamiltonian_rule(state_idx as i32, action, 0.5 * strength);
                }

                // 観測された状態・行動ペアに対するペナルティを劇的に減少させる
                let bin_per_action = self.mwso.dim / self.action_size;
                let start = state_idx * self.mwso.dim + action * bin_per_action;
                for j in 0..bin_per_action {
                    if start + j < self.penalty_matrix.len() {
                        self.penalty_matrix[start + j] *= 0.5;
                    }
                }
            }
        }

        // 3. 状態履歴の更新（エキスパートの「流れ」も模倣する）
        self.input_history.push_back(state_idx);
        if self.input_history.len() > 4 { self.input_history.pop_front(); }
        
        // エキスパートの行動を自身の「最後のアクション」として記録し、
        // 次回の learn 時（もしあれば）に正の実績として扱えるようにする
        self.last_actions = expert_actions.to_vec();
        self.last_state_idx = state_idx;
    }

    pub fn add_wormhole(&mut self, from_action: usize, to_action: usize, strength: f32) {
        let bin_per_action = self.mwso.dim / self.action_size;
        let from_idx = from_action * bin_per_action;
        let to_idx = to_action * bin_per_action;
        self.mwso.add_wormhole(from_idx, to_idx, strength);
    }

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
        for m in &self.action_momentum { file.write_all(&m.to_le_bytes())?; }
        for g in &self.mwso.gravity_field { file.write_all(&g.to_le_bytes())?; }
        
        // input_history の保存
        file.write_all(&(self.input_history.len() as u32).to_le_bytes())?;
        for &s in &self.input_history { file.write_all(&(s as u32).to_le_bytes())?; }
        
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
        let read_u32 = |p: &mut usize| -> u32 { let v = u32::from_le_bytes(buf[*p..*p+4].try_into().unwrap()); *p+=4; v };
        let read_f32 = |p: &mut usize| -> f32 { let v = f32::from_le_bytes(buf[*p..*p+4].try_into().unwrap()); *p+=4; v };
        
        if &buf[0..4] != b"DSYM" { return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid Header")); }
        cur += 4;
        let _version = read_u32(&mut cur);
        let saved_state_size = read_u32(&mut cur) as usize;
        if saved_state_size != self.state_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "state_size mismatch"));
        }

        self.system_temperature = read_f32(&mut cur);
        self.adrenaline = read_f32(&mut cur);
        self.frustration = read_f32(&mut cur);
        self.velocity_trust = read_f32(&mut cur);
        self.morale = read_f32(&mut cur);
        self.patience = read_f32(&mut cur);
        self.exploration_beta = read_f32(&mut cur);
        self.horizon.glutamate_buffer = read_f32(&mut cur);
        
        for f in &mut self.fatigue_map { *f = read_f32(&mut cur); }
        for m in &mut self.action_momentum { *m = read_f32(&mut cur); }
        for g in &mut self.mwso.gravity_field { *g = read_f32(&mut cur); }
        
        let in_hist_len = read_u32(&mut cur) as usize;
        self.input_history.clear();
        for _ in 0..in_hist_len {
            self.input_history.push_back(read_u32(&mut cur) as usize);
        }
        
        let cat_len = read_u32(&mut cur) as usize;
        for _ in 0..cat_len { let _ = read_u32(&mut cur); } // Skip category sizes for now or validate
        
        let nodes_len = read_u32(&mut cur) as usize;
        for i in 0..nodes_len {
            if i < self.nodes.len() {
                self.nodes[i].state = read_f32(&mut cur);
                self.nodes[i].base_decay = read_f32(&mut cur);
            } else {
                let _ = read_f32(&mut cur);
                let _ = read_f32(&mut cur);
            }
        }
        
        let rules_len = read_u32(&mut cur) as usize;
        self.learned_rules.clear();
        for _ in 0..rules_len {
            let s = read_u32(&mut cur) as usize;
            let a = read_u32(&mut cur) as usize;
            let c = read_u32(&mut cur) as usize;
            self.learned_rules.push((s, a, c));
        }

        let mwso_dim = read_u32(&mut cur) as usize;
        if mwso_dim == self.mwso.dim {
            for f in &mut self.mwso.psi_real { *f = read_f32(&mut cur); }
            for f in &mut self.mwso.psi_imag { *f = read_f32(&mut cur); }
            let theta_len = read_u32(&mut cur) as usize;
            for i in 0..theta_len {
                let val = read_f32(&mut cur);
                if i < self.mwso.theta.len() { self.mwso.theta[i] = val; }
            }
        }

        self.last_topology_update_temp = -1.0;
        self.reshape_topology();
        Ok(())
    }
}