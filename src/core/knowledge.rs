use super::singularity::Singularity;

/// 知識ベースのルール定義
pub struct KnowledgeRule {
    pub state_idx: usize,
    pub action_idx: usize,
    pub bias: f32,
}

pub struct Bootstrapper {
    pub rules: Vec<KnowledgeRule>,
}

impl Bootstrapper {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, state_idx: usize, action_idx: usize, bias: f32) {
        self.rules.push(KnowledgeRule {
            state_idx,
            action_idx,
            bias,
        });
    }

    /// Singularity の Qテーブルに知識を注入する
    pub fn apply(&self, singularity: &mut Singularity) {
        for rule in &self.rules {
            let q_idx = rule.state_idx * singularity.action_size + rule.action_idx;
            if q_idx < singularity.q_table.len() {
                // 既存のQ値を知識ベースのバイアスで上書き、または加算
                // ここでは初期推論を強く誘導するため、直接代入に近い形で反映
                singularity.q_table[q_idx] = rule.bias;
            }
        }
        
        // 知識注入後はシステムを一旦安定させるために温度を少し下げる
        singularity.system_temperature *= 0.8;
    }
}
