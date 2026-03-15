use super::horizon::Horizon;
use super::node::Node;
use super::mwso::MWSO;
use super::mwso::ShardedMWSO;
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
    pub sharded_mwso: Option<ShardedMWSO>,
    pub bootstrapper: crate::core::knowledge::Bootstrapper,
    pub active_conditions: Vec<i32>, 
    pub system_temperature: f32,
    pub temperature_locked: bool,
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
    pub penalty_dim: usize,
    pub last_actions: Vec<usize>, 
    pub last_state_idx: usize,
    pub action_momentum: Vec<f32>, 
    pub input_history: VecDeque<usize>, // 入力状態の履歴（流れ）
    pub history: VecDeque<Experience>,
    pub max_history: usize,
    pub learned_rules: Vec<(usize, usize, usize)>, 
    pub penalty_matrix: Vec<f32>, 

    pub empty_penalty: Vec<f32>,
    pub exploration_beta: f32,    
    pub exploration_timer: usize,
    pub current_focus_action: usize,

    pub idx_aggression: usize,
    pub idx_fear: usize,
    pub idx_tactical: usize,
    pub idx_reflex: usize,
}

impl Singularity {
    pub fn new(state_size: usize, category_sizes: Vec<usize>) -> Self {
        let nodes = vec![Node::new(0.5), Node::new(0.4), Node::new(0.3), Node::new(0.3)];
        let total_action_size: usize = category_sizes.iter().sum();

        let shard_threshold = 16; // 16アクション以上はシャード化
        let use_sharding = total_action_size > shard_threshold;

        let (required_dim, penalty_dim) = if use_sharding {
            let p_dim = (total_action_size * 64).next_power_of_two();
            (1024, p_dim)
        } else {
            let dim = (total_action_size * 64).next_power_of_two().max(1024);
            (dim, dim)
        };
        
        Self {
            nodes,
            horizon: Horizon::new(),
            mwso: MWSO::new(required_dim),
            sharded_mwso: if use_sharding {
                Some(ShardedMWSO::new(total_action_size))
            } else {
                None
            },
            bootstrapper: crate::core::knowledge::Bootstrapper::new(),
            active_conditions: Vec::new(),
            system_temperature: 0.5,
            temperature_locked: false,
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
            penalty_dim,
            last_actions: vec![0; category_sizes.len()],
            last_state_idx: 0,
            action_momentum: vec![0.0; total_action_size],
            input_history: VecDeque::with_capacity(8),
            history: VecDeque::with_capacity(32),
            max_history: 15,
            learned_rules: Vec::new(),
            penalty_matrix: vec![0.0; state_size * penalty_dim],
            empty_penalty: vec![0.0; penalty_dim],
            exploration_beta: 0.1, 
            exploration_timer: 0,
            current_focus_action: 0,
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

        let total_dim = self.penalty_dim;
        
        let start = state_idx * total_dim;
        let mut current_penalty_field = self.penalty_matrix[start..start + total_dim].to_vec();

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
        if let Some(ref mut sharded) = self.sharded_mwso {
            sharded.inject_state(state_idx, 1.0, &current_penalty_field);
        } else {
            self.mwso.inject_state(state_idx, 1.0, &current_penalty_field);
        }
        
        // 過去の状態を減衰させながら重畳注入（流れを形成）
        // 2048次元設定では、履歴エネルギーを強めに維持(0.4 -> 0.6)してパスを形成する
        let mut decay = 0.6;
        for &prev_idx in self.input_history.iter().rev() {
        if let Some(ref mut sharded) = self.sharded_mwso {
                // 全シャードに注入するが強度を弱める
                sharded.inject_state(prev_idx, decay * 0.5, &current_penalty_field);
            } else {
                self.mwso.inject_state(prev_idx, decay, &current_penalty_field);
            }
            decay *= 0.6;
            if decay < 0.1 { break; }
        }
        
        // 履歴の更新
        self.input_history.push_back(state_idx);
        if self.input_history.len() > 4 { self.input_history.pop_front(); }
        // ------------------------------------------

        // Background continuous noise is removed for signal clarity.
        // Exploration is now purely through e-greedy and focused irradiation.
        
        // --- Focused Irradiation (Refined: Promising Focus) ---
        if self.system_temperature > 0.3 {
            if self.exploration_timer == 0 {
                // Preview scores without noise to find where the wave is gravitating
                let preview_scores = if let Some(ref mut sharded) = self.sharded_mwso {
                    sharded.get_action_scores(&current_penalty_field)
                } else {
                    self.mwso.get_action_scores(0, self.action_size, 0.0, &current_penalty_field)
                };
                
                // Find top candidates (best or second best)
                let mut best_idx = 0;
                let mut second_idx = 0;
                let mut max_s = -f32::INFINITY;
                let mut second_s = -f32::INFINITY;
                
                for (i, &s) in preview_scores.iter().enumerate() {
                    if s > max_s {
                        second_s = max_s; second_idx = best_idx;
                        max_s = s; best_idx = i;
                    } else if s > second_s {
                        second_s = s; second_idx = i;
                    }
                }
                
                // 70% chance to focus on the leader, 30% on the runner-up to break local optima
                self.current_focus_action = if self.mwso.next_rng() < 0.7 { best_idx } else { second_idx };
                
                // Timer scales with sqrt(dim) to allow interference formation in high dimensions
                let dim_ratio = (self.mwso.dim as f32 / 1024.0).sqrt();
                self.exploration_timer = (15.0 * dim_ratio) as usize; 
            }
            
            // Linear scaling for strength: higher dims need linear boost to combat energy diffusion
            let dim_boost = self.mwso.dim as f32 / 1024.0;
            let strength = 0.1 * self.system_temperature * dim_boost; 
            
            if let Some(ref mut sharded) = self.sharded_mwso {
                sharded.illuminate_bin(self.current_focus_action, strength);
            } else {
                self.mwso.illuminate_bin(self.current_focus_action, self.action_size, strength);
            }
            self.exploration_timer -= 1;
        }
        
        if let Some(ref mut sharded) = self.sharded_mwso {
            sharded.step_core(0.1, speed_boost, focus_factor, self.system_temperature, &current_penalty_field);
        } else {
            self.mwso.step_core(0.1, speed_boost, focus_factor, self.system_temperature, &current_penalty_field);
        }

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
        let mwso_scores = if let Some(ref mut sharded) = self.sharded_mwso {
            // 1. シャード全体から全アクションのスコアを一気に取得
            // ※この内部で各シャードの get_action_scores が並列（または順次）に走る
            let all_scores = sharded.get_action_scores(penalty_field);
            
            // 2. 必要な範囲（カテゴリ）だけを切り出す
            // offset と size が total_dim (2048) を超えないよう安全にスライス
            let end = (offset + size).min(all_scores.len());
            all_scores[offset..end].to_vec()
        } else {
            // 従来の 1024次元単体モード
            self.mwso.get_action_scores(offset, size, 0.0, penalty_field)
        };
        let active_resonance = self.bootstrapper.calculate_resonance_field(&self.active_conditions, self.action_size);

        let mut candidate_scores = Vec::with_capacity(size);

        for i in 0..size {
            let mut knowledge_field = 0.0;
            if let Some(s) = active_resonance[offset + i] {
                if s < -0.9 { knowledge_field = -100.0; } 
                else { knowledge_field = s * 5.0; }
            }
            
            let mwso_component = mwso_scores[i];
            let internal_field = self.learned_rules.iter()
                .find(|r| r.0 == self.last_state_idx && r.1 == offset + i)
                .map(|r| (r.2 as f32 * 1.0).min(5.0)).unwrap_or(0.0);

            if let Some(rule) = self.bootstrapper.rules.iter().find(|r| r.condition_id == self.last_state_idx as i32 && r.target_action == offset + i) {
                knowledge_field += rule.strength * 5.0;
            }

            let neuron_boost = match i {
                0 => self.nodes[self.idx_aggression].state * 0.5,
                1 => self.nodes[self.idx_fear].state * 0.3,
                _ => 0.0,
            };
            
            let momentum_boost = self.action_momentum[offset + i] * 1.0;
            let fatigue_penalty = self.fatigue_map[offset + i] * 2.0;
            
            let total_score = mwso_component + internal_field + knowledge_field + neuron_boost + momentum_boost - fatigue_penalty + (self.morale * 0.1);
            candidate_scores.push((i, total_score));
        }

        // --- Top-k Softmax Sampling ---
        // 1. Sort by score descending
        candidate_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 2. Take Top-k (k=3 or size if smaller)
        let k = 3.min(size);
        let top_k = &candidate_scores[..k];

        // 3. Compute Softmax probabilities over Top-k
        // Probability depends on inverse temperature
        let beta = (1.0 / self.system_temperature.max(0.05)) * 2.0;
        let mut probs = Vec::with_capacity(k);
        let max_s = top_k[0].1;
        let mut sum_exp = 0.0;

        for &(_, s) in top_k {
            let p = ((s - max_s) * beta).exp(); // subtract max for numerical stability
            probs.push(p);
            sum_exp += p;
        }

        // 4. Weighted Random Sample from Top-k
        let mut r = self.mwso.next_rng() * sum_exp;
        for i in 0..k {
            r -= probs[i];
            if r <= 0.0 {
                return top_k[i].0;
            }
        }
        top_k[0].0
    }

    pub fn learn(&mut self, reward: f32) {
        let mut discount = 1.0;
        let gamma = 0.9;

        for exp in self.history.iter().rev() {
            let discounted_reward = reward * discount;
            if let Some(ref mut sharded) = self.sharded_mwso {
                sharded.adapt(discounted_reward, &exp.actions, self.system_temperature);

                // シャード間トンネルの学習
                if discounted_reward > 0.1 && !sharded.shards.is_empty() {
                    let state_shard_idx = exp.state_idx % sharded.shards.len();
                    for &action_idx in &exp.actions {
                        let (action_shard_idx, local_action) = sharded.shard_for_action(action_idx);
                        if state_shard_idx != action_shard_idx {
                            // 状態とアクションの担当シャードが違う場合、トンネルを強化
                            let strength = (0.05 * discounted_reward).min(0.1);
                            sharded.add_or_strengthen_tunnel(state_shard_idx, action_shard_idx, exp.state_idx, local_action, strength);
                        }
                    }
                }
            } else {
                self.mwso.adapt(discounted_reward, &exp.actions, self.system_temperature, self.action_size);
            }

            if self.active_conditions.is_empty() {
                let state = exp.state_idx;
                let action = exp.actions[0];
                let dim_stability = (1024.0 / self.mwso.dim as f32).sqrt().min(1.0);

                if discounted_reward > 1.2 {
                    if let Some(rule) = self.learned_rules.iter_mut().find(|r| r.0 == state && r.1 == action) {
                        rule.2 += 1;
                    } else {
                        self.learned_rules.push((state, action, 1));
                    }
                    let penalty_dim = self.penalty_dim;
                    let bin_per_action = penalty_dim / self.action_size;
                    let start = state * penalty_dim + action * bin_per_action;
                    // 成功時にペナルティを消す力も次元数で調整
                    for j in 0..bin_per_action { self.penalty_matrix[start + j] *= 0.5 + 0.4 * (1.0 - dim_stability); }
                } else if discounted_reward < 0.0 {
                    let penalty_dim = self.penalty_dim;
                    let bin_per_action = penalty_dim / self.action_size;
                    let start = state * penalty_dim + action * bin_per_action;
                    for j in 0..bin_per_action { 
                        // 失敗時のペナルティ注入を次元数に応じて薄める
                        let p_add = (discounted_reward.abs() * 2.0 * dim_stability).min(10.0);
                        self.penalty_matrix[start + j] = (self.penalty_matrix[start + j] + p_add).min(10.0); 
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
        if !self.temperature_locked {
            // 高次元ほど「なまし（Annealing）」を長く保つ
            let dim_inertia = (self.mwso.dim as f32 / 1024.0).sqrt().max(1.0);
            
            if reward > 0.0 {
                let cooling_rate = (0.8 + (reward * 0.1).min(0.15)) / dim_inertia; 
                let mut next_temp = self.system_temperature * (1.0 - cooling_rate * 0.2) - reward * 0.05 / dim_inertia;
                
                // --- Stability Guard (Rhyd Feedback) ---
                // If resonance is high, force cool to stabilize the pattern and prevent overshoot
                let rhyd = if let Some(ref sharded) = self.sharded_mwso { sharded.calculate_rhyd() } else { self.mwso.calculate_rhyd() };
                if rhyd > 5.0 {
                    next_temp *= 0.7; // Rapid stabilization
                }
                
                // IPRが低い（確信している）時は、冷却を加速して 0 に近づける
                let ipr = if let Some(ref sharded) = self.sharded_mwso { sharded.calculate_ipr() } else { self.mwso.calculate_ipr() };
                let ipr_threshold = if self.sharded_mwso.is_some() {
                    let num_shards = self.sharded_mwso.as_ref().unwrap().num_shards();
                    25.0 * num_shards as f32  // 2シャード→50.0
                } else {
                    25.0
                };
                
                if ipr < ipr_threshold { next_temp *= 0.5; }
                
                self.system_temperature = next_temp.max(0.01);
            } else {
                // IPR（波動の集中度）をチェック。集中している(IPRが低い)ほど、失敗に動じない。
                let ipr = if let Some(ref sharded) = self.sharded_mwso { sharded.calculate_ipr() } else { self.mwso.calculate_ipr() };
                let confidence_guard = (1.0 - (10.0 / ipr.max(10.0))).clamp(0.1, 1.0);
                
                // 確信度が高い（IPRが低い）時は、加熱（温度上昇）を最大 90% カットする
                let heating = (td_error * 0.3 / dim_inertia).min(1.0) * confidence_guard; 
                self.system_temperature = (self.system_temperature + heating).min(2.0);
            }
        }

        let urgency = ((reward + penalty) * 5.0).min(1.0);
        
        match &mut self.sharded_mwso {
            Some(sharded) => {
                sharded.inject_state(0, reward, &self.empty_penalty);
                sharded.inject_state(1, -penalty, &self.empty_penalty);
                sharded.step_core(0.05, 0.0, 0.0, self.system_temperature, &self.empty_penalty);
            },
            None => {
                // In non-sharded mode, mwso.dim and penalty_dim are the same.
                self.mwso.inject_state(0, reward, &self.empty_penalty);
                self.mwso.inject_state(1, -penalty, &self.empty_penalty);
                self.mwso.step_core(0.05, 0.0, 0.0, self.system_temperature, &self.empty_penalty);
            }
        }

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
        self.mwso.step_core(0.1, 0.0, 0.0, self.system_temperature, &vec![0.0; self.mwso.dim]);
        let current_states: Vec<f32> = self.nodes.iter().map(|n| n.state).collect();
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let input = input_signals.get(i).cloned().unwrap_or(0.0);
            node.update(input, urgency, self.system_temperature, &current_states);
        }
    }

    pub fn set_neuron_state(&mut self, idx: usize, state: f32) {
        if let Some(node) = self.nodes.get_mut(idx) { node.state = state.clamp(0.0, 1.0); }
    }

    pub fn get_resonance_density(&self) -> f32 {
        if let Some(ref sharded) = self.sharded_mwso {
            sharded.calculate_rhyd() // 全シャードの平均値を取得
        } else {
            self.mwso.calculate_rhyd()
        }
    }

    pub fn calculate_current_ipr(&self) -> f32 {
        if let Some(ref sharded) = self.sharded_mwso {
            sharded.calculate_ipr()
        } else {
            self.mwso.calculate_ipr()
        }
    }

    /// 逆強化学習: 行動から動機を逆算する
    /// エキスパートの行動を観測し、それを引き起こす「ハミルトニアン場（動機）」を内省的に生成する
    pub fn observe_expert(&mut self, state_idx: usize, expert_actions: &[usize], strength: f32) {
        // 1. 位相の同調（模倣位相ロック）
        for &action in expert_actions {
            if let Some(ref mut sharded) = self.sharded_mwso {
                // ShardedMWSOのalign_to_actionを呼び出す
                sharded.align_to_action(action, strength);
            } else {
                self.mwso.align_to_action(action, strength, self.action_size);
            }
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
                let penalty_dim = self.penalty_matrix.len() / self.state_size;
                let bin_per_action = penalty_dim / self.action_size;
                let start = state_idx * self.penalty_dim + action * bin_per_action;
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
        file.write_all(&13u32.to_le_bytes())?; 
        file.write_all(&(self.state_size as u32).to_le_bytes())?;
        file.write_all(&self.system_temperature.to_le_bytes())?;
        file.write_all(&(if self.temperature_locked { 1u32 } else { 0u32 }).to_le_bytes())?;
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
        let version = read_u32(&mut cur);
        let saved_state_size = read_u32(&mut cur) as usize;
        if saved_state_size != self.state_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "state_size mismatch"));
        }

        self.system_temperature = read_f32(&mut cur);
        if version >= 13 {
            self.temperature_locked = read_u32(&mut cur) != 0;
        } else {
            self.temperature_locked = false;
        }
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

    pub fn get_raw_scores(&mut self, action_size: usize) -> Vec<f32> {
        if let Some(ref mut sharded) = self.sharded_mwso {
            sharded.get_action_scores(&vec![0.0; self.penalty_dim])
        } else {
            self.mwso.get_action_scores(0, action_size, 0.0, &vec![0.0; self.mwso.dim])
        }
    }
}