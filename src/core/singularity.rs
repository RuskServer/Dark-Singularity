// のロジックを移植
use super::node::Node;
use super::horizon::Horizon;

pub struct Singularity {
    pub nodes: Vec<Node>,
    pub horizon: Horizon,
    pub system_temperature: f32,
    pub adrenaline: f32,
    pub frustration: f32,
    pub velocity_trust: f32,
    pub fatigue_map: [f32; 8],
    pub morale: f32,
    pub patience: f32,
    pub q_table: Vec<f32>, // 512状態 * 8行動 のフラット配列
    pub last_action_idx: usize,
    pub last_state_idx: usize,
    
    // 特定の役割を持つノードのインデックス
    pub idx_aggression: usize,
    pub idx_fear: usize,
    pub idx_tactical: usize,
    pub idx_reflex: usize,
}

impl Singularity {
    pub fn new() -> Self {
        // 主要な4つのニューロンを初期化
        let nodes = vec![
            Node::new(0.5), // 0: Aggression (攻撃)
            Node::new(0.4), // 1: Fear (恐怖)
            Node::new(0.3), // 2: Tactical (戦術)
            Node::new(0.3), // 3: Reflex (反射)
        ];

        Self {
            nodes,
            horizon: Horizon::new(),
            system_temperature: 0.5,
            adrenaline: 0.0,
            frustration: 0.0,
            velocity_trust: 1.0,
            fatigue_map: [0.0; 8],
            morale: 1.0,
            patience: 1.0,
            // 512状態 * 8行動 = 4096要素のQテーブルを0.0で初期化
            q_table: vec![0.0; 512 * 8], 
            last_action_idx: 4, // OBSERVE
            last_state_idx: 0,
            idx_aggression: 0,
            idx_fear: 1,
            idx_tactical: 2,
            idx_reflex: 3,
        }
    }
    
    pub fn select_action(&mut self, state_idx: usize) -> usize {
        self.last_state_idx = state_idx;
        let mut best_action = 4; // Default: OBSERVE
        let mut max_score = -f32::INFINITY;

        let base_offset = state_idx * 8;

        for action_idx in 0..8 {
            // Q値 + ニューロンの活性度（Aggressionなど）を統合してスコアリング
            let q_value = self.q_table[base_offset + action_idx];
            
            // 例: Action 0 (ATTACK) なら Aggression ノードの状態を重畳
            let neuron_boost = match action_idx {
                0 => self.nodes[self.idx_aggression].state * 1.5,
                1 => self.nodes[self.idx_fear].state * 1.2,
                _ => 0.0,
            };

            let score = q_value + neuron_boost + (self.morale * 0.2);

            if score > max_score {
                max_score = score;
                best_action = action_idx;
            }
        }

        self.last_action_idx = best_action;
        best_action
    }

    /// [TQH-Learning] 学習ロジックの完成版
    pub fn learn(&mut self, reward: f32) {
        let state_offset = self.last_state_idx * 8;
        let current_q = self.q_table[state_offset + self.last_action_idx];
        
        // 予測誤差 (TD Error)
        let td_error = reward - current_q;

        // TQH: 誤差が温度を上げ、熱が学習率(Alpha)をブーストする
        let learning_rate = 0.1 * (1.0 + self.system_temperature);
        
        // Q値の更新
        self.q_table[state_offset + self.last_action_idx] += learning_rate * td_error;

        // 経験の消化（温度変化とトポロジー再編）を呼び出し
        self.digest_experience(td_error, reward, if reward < 0.0 { reward.abs() } else { 0.0 });
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
        
        self.reshape_topology();
        
        let all_indices: Vec<usize> = (0..self.nodes.len()).collect();
        self.horizon.regulate(self.system_temperature, &all_indices, &mut self.nodes);
    }

    // TQHによる相転移ロジック
    /// [TQH-DSR] システム温度に基づいた動的トポロジー再編
    /// Java版 LiquidBrain.reshapeTopology() の完全移行版
    pub fn reshape_topology(&mut self) {
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
        self.nodes[from].synapses.push(super::node::Synapse { target_id: to, weight });
    }
}