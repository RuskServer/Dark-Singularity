// src/core/knowledge.rs

/// ハミルトニアン・ルール: 波動状態に対する「外場」としての知識
pub struct HamiltonianRule {
    /// 発動条件のインデックス (Java側からの指定を容易にするため ID制に)
    /// 実装例: 0=HP低, 1=敵至近, 2=弾薬少 など
    pub condition_id: i32,
    /// 誘導したいアクションのインデックス
    pub target_action: usize,
    /// 知識の強制力 (resonance_strength)
    pub strength: f32,
}

pub struct Bootstrapper {
    pub rules: Vec<HamiltonianRule>,
}

impl Bootstrapper {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_hamiltonian_rule(&mut self, condition_id: i32, target_action: usize, strength: f32) {
        self.rules.push(HamiltonianRule {
            condition_id,
            target_action,
            strength,
        });
    }

    /// 現在の状況（外部から与えられた条件フラグ群）に基づき、
    /// MWSOの各アクションに対する「外場（Resonance Field）」を計算する
    /// 未定義のアクションに対しては 0.0 ではなく、None に相当する値を返せるようにし、
    /// 知識が「ない」状態と「0である」状態を区別する
    pub fn calculate_resonance_field(&self, active_conditions: &[i32], action_size: usize) -> Vec<Option<f32>> {
        let mut field = vec![None; action_size];
        for rule in &self.rules {
            if active_conditions.contains(&rule.condition_id) {
                if rule.target_action < action_size {
                    let current = field[rule.target_action].unwrap_or(0.0);
                    field[rule.target_action] = Some(current + rule.strength);
                }
            }
        }
        field
    }
}
