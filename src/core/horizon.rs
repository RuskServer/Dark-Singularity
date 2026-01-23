use super::node::Node;

pub struct Horizon {
    pub glutamate_buffer: f32,
    pub homeostatic_threshold: f32,
}

impl Horizon {
    pub fn new() -> Self {
        Self {
            glutamate_buffer: 0.0,
            homeostatic_threshold: 1.8,
        }
    }

    pub fn regulate(&mut self, system_temp: f32, node_indices: &[usize], nodes: &mut [Node]) {
        // 1. 総活動量の計測
        let total_activity: f32 = node_indices.iter().map(|&i| nodes[i].state).sum();

        // 2. バッファの蓄積と減衰
        self.glutamate_buffer += total_activity * 0.1;
        self.glutamate_buffer *= 0.92;

        // 3. 恒常性スケーリング
        if system_temp > 1.0 && (total_activity > self.homeostatic_threshold || self.glutamate_buffer > 2.0) {
            for &i in node_indices {
                if nodes[i].state > 0.5 {
                    nodes[i].apply_inhibition(0.15);
                }
            }
        }
    }

    pub fn get_intervention_level(&self) -> f32 {
        (self.glutamate_buffer / 3.0).min(1.0)
    }
}