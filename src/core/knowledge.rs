use super::singularity::Singularity;

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

    /// MWSOのパラメータ(theta)に知識を注入する
    pub fn apply(&self, singularity: &mut Singularity) {
        for rule in &self.rules {
            // MWSOの投影は action_idx * 16 + j で定義されている
            // 指定されたアクションの投影重みをバイアスに基づいて強化する
            for j in 0..16 {
                let theta_idx = (rule.action_idx * 16 + j) % 512;
                // バイアス分だけthetaをシフトさせる
                singularity.mwso.theta[theta_idx] += rule.bias * 0.1;
                singularity.mwso.theta[theta_idx] = singularity.mwso.theta[theta_idx].clamp(-2.0, 2.0);
            }
        }
        
        // 知識注入後はシステム温度を下げて安定させる
        singularity.system_temperature *= 0.8;
    }
}