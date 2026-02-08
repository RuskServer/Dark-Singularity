// のロジックを移植
pub struct Synapse {
    pub target_id: usize, // インデックスによる直接参照
    pub weight: f32,
}

pub struct Node {
    pub state: f32,
    pub base_decay: f32,
    pub synapses: Vec<Synapse>,
}

impl Node {
    pub fn new(initial_decay: f32) -> Self {
        Self {
            state: 0.0,
            base_decay: initial_decay,
            synapses: Vec::new(),
        }
    }

    /// [TQH Update] システム温度を考慮した更新ロジック
    pub fn update(&mut self, input: f32, urgency: f32, system_temp: f32, node_states: &[f32]) {
        let mut synaptic_input = input;
    
        // シナプス入力の計算 (node_states からインデックスで取得)
        for synapse in &self.synapses {
            if let Some(&state) = node_states.get(synapse.target_id) {
                synaptic_input += state * synapse.weight;
            }
        }
        
        // 自己回帰的な特性の付与
        synaptic_input += self.state * 0.1;
    
        // TQH: 温度による流動性(alpha)の計算
        let thermal_effect = (system_temp * 0.4).max(0.0);
        let alpha = (self.base_decay + (urgency * (1.0 - self.base_decay)) + thermal_effect)
            .clamp(0.01, 1.0);
    
        // 状態の更新
        self.state += alpha * (synaptic_input - self.state);
        self.state = self.state.clamp(0.0, 1.0);
    }

    pub fn apply_inhibition(&mut self, dampening_factor: f32) {
        self.state -= self.state * dampening_factor;
        self.state = self.state.max(0.0);
    }
}