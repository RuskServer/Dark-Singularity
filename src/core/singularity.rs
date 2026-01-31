// のロジックを移植
use super::horizon::Horizon;
use super::node::Node;
use std::fs::File;
use std::io::{self, Read, Write};

pub struct Singularity {
    pub nodes: Vec<Node>,
    pub horizon: Horizon,
    pub system_temperature: f32,
    pub last_topology_update_temp: f32,
    pub adrenaline: f32,
    pub frustration: f32,
    pub velocity_trust: f32,
    pub fatigue_map: Vec<f32>,
    pub morale: f32,
    pub patience: f32,
    pub q_table: Vec<f32>,     // 状態数 * 合計アクション数
    pub category_sizes: Vec<usize>, // 追加: 各カテゴリーのアクション数
    pub action_size: usize,    // 合計アクション数
    pub state_size: usize,
    pub last_actions: Vec<usize>, // 最後に選択されたアクション群
    pub last_state_idx: usize,

    pub visit_counts: Vec<u32>,    // カウントベース探査用: 状態数 * 合計アクション数
    pub exploration_beta: f32,    // 探査の強さ

    pub idx_aggression: usize,
    pub idx_fear: usize,
    pub idx_tactical: usize,
    pub idx_reflex: usize,
}

impl Singularity {
    pub fn new(state_size: usize, category_sizes: Vec<usize>) -> Self {
        let nodes = vec![
            Node::new(0.5), // Aggression
            Node::new(0.4), // Fear
            Node::new(0.3), // Tactical
            Node::new(0.3), // Reflex
        ];

        let total_action_size: usize = category_sizes.iter().sum();

        Self {
            nodes,
            horizon: Horizon::new(),
            system_temperature: 0.5,
            last_topology_update_temp: -1.0,
            adrenaline: 0.0,
            frustration: 0.0,
            velocity_trust: 1.0,
            fatigue_map: vec![0.0; total_action_size],
            morale: 1.0,
            patience: 1.0,
            q_table: vec![0.0; state_size * total_action_size],
            category_sizes: category_sizes.clone(),
            action_size: total_action_size,
            state_size,
            last_actions: vec![0; category_sizes.len()],
            last_state_idx: 0,
            visit_counts: vec![0; state_size * total_action_size],
            exploration_beta: 0.1, // デフォルトの探査定数
            idx_aggression: 0,
            idx_fear: 1,
            idx_tactical: 2,
            idx_reflex: 3,
        }
    }

    /// 各カテゴリーから最適なアクションを1つずつ選択する
    pub fn select_actions(&mut self, state_idx: usize) -> Vec<i32> {
        self.last_state_idx = state_idx;
        let mut results = Vec::with_capacity(self.category_sizes.len());
        let mut current_offset = 0;

        for (cat_idx, &size) in self.category_sizes.iter().enumerate() {
            let best_action_in_cat = self.get_best_in_range(state_idx, current_offset, size);
            self.last_actions[cat_idx] = current_offset + best_action_in_cat;
            results.push(best_action_in_cat as i32);
            current_offset += size;
        }

        results
    }

    fn get_best_in_range(&self, state_idx: usize, offset: usize, size: usize) -> usize {
        let mut best = 0;
        let mut max_score = -f32::INFINITY;
        let base_offset = state_idx * self.action_size + offset;

        for i in 0..size {
            let q_idx = base_offset + i;
            let q_value = self.q_table[q_idx];
            
            // カウントベース探査ボーナス: beta / sqrt(count + 1)
            let count = self.visit_counts[q_idx] as f32;
            let exploration_bonus = self.exploration_beta / (count + 1.0).sqrt();

            // ニューロンによる補正
            let neuron_boost = match i {
                0 => self.nodes[self.idx_aggression].state * 1.5,
                1 => self.nodes[self.idx_fear].state * 1.2,
                _ => 0.0,
            };
            
            let score = q_value + exploration_bonus + neuron_boost + (self.morale * 0.1);

            if score > max_score {
                max_score = score;
                best = i;
            }
        }
        best
    }

    /// 最後に選択された全アクションに対して一括で学習を実行
    pub fn learn(&mut self, reward: f32) {
        let learning_rate = 0.1 * (1.0 + self.system_temperature);
        let mut total_td_error = 0.0;

        for &action_idx in &self.last_actions {
            let q_idx = self.last_state_idx * self.action_size + action_idx;
            
            // カウントを更新
            self.visit_counts[q_idx] = self.visit_counts[q_idx].saturating_add(1);

            let current_q = self.q_table[q_idx];
            let td_error = reward - current_q;
            self.q_table[q_idx] += learning_rate * td_error;
            total_td_error += td_error.abs();
        }

        let avg_td_error = total_td_error / (self.last_actions.len() as f32).max(1.0);

        self.digest_experience(
            avg_td_error,
            reward,
            if reward < 0.0 { reward.abs() } else { 0.0 },
        );
    }

    /// 経験の消化 (TQH平衡計算)
    pub fn digest_experience(&mut self, td_error: f32, reward: f32, penalty: f32) {
        self.system_temperature += td_error.abs() * 0.25;
        if reward > 0.0 {
            self.system_temperature -= reward * 0.45;
        }
        self.system_temperature = (self.system_temperature.clamp(0.0, 2.0)) * 0.94;

        let urgency = ((reward + penalty) * 5.0).min(1.0);

        // 状態更新も update_all_nodes と同じロジックで行う
        let current_states: Vec<f32> = self.nodes.iter().map(|n| n.state).collect();
        for node in &mut self.nodes {
            node.update(0.0, urgency, self.system_temperature, &current_states);
        }

        // 最適化: 緊急性が高いか、システム温度が大きく変化した場合のみトポロジーを再構築
        let temp_diff = (self.system_temperature - self.last_topology_update_temp).abs();
        if urgency > 0.5 || temp_diff > 0.05 {
            self.reshape_topology();
        }

        let all_indices: Vec<usize> = (0..self.nodes.len()).collect();
        self.horizon
            .regulate(self.system_temperature, &all_indices, &mut self.nodes);
    }

    // TQHによる相転移ロジック
    /// [TQH-DSR] システム温度に基づいた動的トポロジー再編
    /// Java版 LiquidBrain.reshapeTopology() の完全移行版
    pub fn reshape_topology(&mut self) {
        // 更新時の温度を記録
        self.last_topology_update_temp = self.system_temperature;

        // 1. 全シナプスの物理的切断 (一旦全クリア)
        // Javaの disconnect() ループを Vec::clear() で高速化
        for node in &mut self.nodes {
            node.synapses.clear();
        }

        let glia_intervention = self.horizon.get_intervention_level();

        // 2. 相転移ロジック (System Temperature による分岐)

        // --- GAS (気体) 状態: 1.2+ ---
        // 高エネルギー・暴走状態。反射(Reflex)と直感(Aggression)が直結。
        if self.system_temperature > 1.2 {
            // 攻撃衝動を反射に直結（最速の反応）
            self.connect(self.idx_aggression, self.idx_reflex, 2.0);

            // アストロサイト（グリア）が過剰発火を検知した場合
            if glia_intervention > 0.6 {
                // 攻撃回路を強制遮断し、恐怖(Fear)による回避を最優先
                self.nodes[self.idx_aggression].synapses.clear();
                self.connect(self.idx_reflex, self.idx_fear, 1.5);
                self.connect(self.idx_fear, self.idx_reflex, 1.2);
            }
        }
        // --- SOLID (固体) 状態: 0.0 - 0.3 ---
        // 低エネルギー・結晶化状態。戦術(Tactical)に基づいた精密な最適化。
        else if self.system_temperature < 0.3 {
            // 疲労が少ない場合のみ、戦術から攻撃へ正確な指令を出す
            if self.fatigue_map[0] < 0.5 {
                self.connect(self.idx_tactical, self.idx_aggression, 1.2);
            }

            // 結晶化しているため、無駄な反射を抑制し、精密射撃や距離維持に特化
            self.connect(self.idx_tactical, self.idx_reflex, 0.5);
        }
        // --- LIQUID (液体) 状態: 0.3 - 1.2 ---
        // 平常・適応状態。環境に合わせて柔軟にトポロジーを構築。
        else {
            // 基本的な戦術ループ
            self.connect(self.idx_tactical, self.idx_reflex, 1.0);

            // アドレナリン（情動）が高い場合、攻撃性をブースト
            if self.adrenaline > 0.5 {
                self.connect(self.idx_aggression, self.idx_reflex, 1.5);
            } else {
                // 通常時は均衡を維持
                self.connect(self.idx_fear, self.idx_reflex, 0.8);
                self.connect(self.idx_aggression, self.idx_tactical, 0.7);
            }

            // フラストレーション（停滞）が溜まっている場合、突破口としてタクティカルを強化
            if self.frustration > 0.7 {
                self.connect(self.idx_tactical, self.idx_aggression, 1.8);
            }
        }

        // 3. Morphogenic Topology: 疲労マップ（EMDA 1. Elastic Q）を結合強度に反映
        // 特定の経路を使いすぎると、自動的に結合が弱まる「飽き」の実装
        self.apply_elastic_fatigue();
    }

    /// [EMDA-1] 弾性Q学習: 疲労によるシナプス強度の動的減衰
    fn apply_elastic_fatigue(&mut self) {
        for (idx, fatigue) in self.fatigue_map.iter().enumerate() {
            if *fatigue > 0.8 {
                // 該当するインデックスに関連するニューロンの重みを強制下方修正
                // これにより「同じ行動の繰り返し」を防ぎ、揺らぎ（フェイント）を生む
                for node in &mut self.nodes {
                    for synapse in &mut node.synapses {
                        if synapse.target_id == idx {
                            synapse.weight *= 0.5;
                        }
                    }
                }
            }
        }
    }

    pub fn update_all_nodes(&mut self, input_signals: &[f32], urgency: f32) {
        // 1. 全ノードの「現在の状態」だけを抜き出した配列を作る
        let current_states: Vec<f32> = self.nodes.iter().map(|n| n.state).collect();

        // 2. その配列を参照として Node::update に渡す
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let input = input_signals.get(i).cloned().unwrap_or(0.0);
            // ここ！ &self.nodes ではなく &current_states を渡す
            node.update(input, urgency, self.system_temperature, &current_states);
        }
    }

    fn connect(&mut self, from: usize, to: usize, weight: f32) {
        self.nodes[from].synapses.push(super::node::Synapse {
            target_id: to,
            weight,
        });
    }

    // --- Persistance (Custom Binary Format .dsym) ---
    /*
        Format Spec:
        [Header] "DSYM" (4 bytes)
        [Version] u32 (4 bytes)
        [SystemTemp] f32
        [Adrenaline] f32
        [Frustration] f32
        [VelocityTrust] f32
        [Morale] f32
        [Patience] f32
        [FatigueMap] f32 * action_size
        [CategorySizes] count: u32 -> [size: u32] * count
        [QTable] f32 * (state_size * action_size)
        [Nodes] count: u32 -> [state: f32, base_decay: f32] * count
    */

    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let mut file = File::create(path)?;

        // Header
        file.write_all(b"DSYM")?;
        file.write_all(&4u32.to_le_bytes())?; // Version 4 (Meta validation & Horizon state)

        // Meta data
        file.write_all(&(self.state_size as u32).to_le_bytes())?;

        // Parameters
        file.write_all(&self.system_temperature.to_le_bytes())?;
        file.write_all(&self.adrenaline.to_le_bytes())?;
        file.write_all(&self.frustration.to_le_bytes())?;
        file.write_all(&self.velocity_trust.to_le_bytes())?;
        file.write_all(&self.morale.to_le_bytes())?;
        file.write_all(&self.patience.to_le_bytes())?;
        file.write_all(&self.exploration_beta.to_le_bytes())?;

        // Horizon state (New in V4)
        file.write_all(&self.horizon.glutamate_buffer.to_le_bytes())?;

        // Arrays
        for f in &self.fatigue_map {
            file.write_all(&f.to_le_bytes())?;
        }

        // Visit Counts
        for &c in &self.visit_counts {
            file.write_all(&c.to_le_bytes())?;
        }

        // Category Sizes
        file.write_all(&(self.category_sizes.len() as u32).to_le_bytes())?;
        for &s in &self.category_sizes {
            file.write_all(&(s as u32).to_le_bytes())?;
        }

        for q in &self.q_table {
            file.write_all(&q.to_le_bytes())?;
        }

        // Nodes
        file.write_all(&(self.nodes.len() as u32).to_le_bytes())?;
        for node in &self.nodes {
            file.write_all(&node.state.to_le_bytes())?;
            file.write_all(&node.base_decay.to_le_bytes())?;
        }

        Ok(())
    }

    pub fn load_from_file(&mut self, path: &str) -> io::Result<()> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let mut cursor = 0;

        let read_u32 = |pos: &mut usize| -> io::Result<u32> {
            if *pos + 4 > buffer.len() { return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF")); }
            let val = u32::from_le_bytes(buffer[*pos..*pos + 4].try_into().unwrap());
            *pos += 4;
            Ok(val)
        };
        let read_f32 = |pos: &mut usize| -> io::Result<f32> {
            if *pos + 4 > buffer.len() { return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF")); }
            let val = f32::from_le_bytes(buffer[*pos..*pos + 4].try_into().unwrap());
            *pos += 4;
            Ok(val)
        };

        if buffer.len() < 4 || &buffer[0..4] != b"DSYM" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic bytes"));
        }
        cursor += 4;

        let version = read_u32(&mut cursor)?;
        
        // Version 4+: state_size check
        if version >= 4 {
            let loaded_state_size = read_u32(&mut cursor)? as usize;
            if loaded_state_size != self.state_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Incompatible state size: expected {}, found {}", self.state_size, loaded_state_size)
                ));
            }
        }

        self.system_temperature = read_f32(&mut cursor)?;
        self.adrenaline = read_f32(&mut cursor)?;
        self.frustration = read_f32(&mut cursor)?;
        self.velocity_trust = read_f32(&mut cursor)?;
        self.morale = read_f32(&mut cursor)?;
        self.patience = read_f32(&mut cursor)?;

        if version >= 3 {
            self.exploration_beta = read_f32(&mut cursor)?;
        }

        // Horizon state
        if version >= 4 {
            self.horizon.glutamate_buffer = read_f32(&mut cursor)?;
        }

        if version >= 2 {
            // Fatigue map load
            for i in 0..self.action_size {
                self.fatigue_map[i] = read_f32(&mut cursor)?;
            }

            if version >= 3 {
                // visit_counts の読み込み (ファイル側のカウントを優先)
                for i in 0..self.visit_counts.len() {
                    self.visit_counts[i] = read_u32(&mut cursor)?;
                }
            }

            let cat_count = read_u32(&mut cursor)? as usize;
            let mut loaded_cats = Vec::with_capacity(cat_count);
            for _ in 0..cat_count {
                loaded_cats.push(read_u32(&mut cursor)? as usize);
            }
            
            // カテゴリー構成が一致する場合のみQテーブルをロード
            if loaded_cats == self.category_sizes {
                for i in 0..self.q_table.len() {
                    self.q_table[i] = read_f32(&mut cursor)?;
                }
            } else {
                // 不一致の場合はスキップ (V4であれば state_size が一致しているので計算可能)
                cursor += (self.state_size * loaded_cats.iter().sum::<usize>()) * 4;
            }
        }

        // Nodes
        let node_count = read_u32(&mut cursor)? as usize;
        for i in 0..node_count.min(self.nodes.len()) {
            self.nodes[i].state = read_f32(&mut cursor)?;
            self.nodes[i].base_decay = read_f32(&mut cursor)?;
        }

        self.last_topology_update_temp = -1.0;
        self.reshape_topology();

        Ok(())
    }


}
