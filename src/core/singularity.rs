use super::horizon::Horizon;
use super::node::Node;
use super::mwso::MWSO;
use std::fs::File;
use std::io::{self, Read, Write};

pub struct Singularity {
    pub nodes: Vec<Node>,
    pub horizon: Horizon,
    pub mwso: MWSO,
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

    pub exploration_beta: f32,    

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
            mwso: MWSO::new(),
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
            exploration_beta: 0.1, 
            idx_aggression: 0,
            idx_fear: 1,
            idx_tactical: 2,
            idx_reflex: 3,
        }
    }

    pub fn select_actions(&mut self, state_idx: usize) -> Vec<i32> {
        self.last_state_idx = state_idx;
        
        let mut input_wave = vec![0.0; 256];
        if state_idx < 256 {
            input_wave[state_idx] = 1.0;
        }
        self.mwso.step(&input_wave, 0.1);

        let mut results = Vec::with_capacity(self.category_sizes.len());
        let mut current_offset = 0;

        for (cat_idx, &size) in self.category_sizes.iter().enumerate() {
            let best_action_in_cat = self.get_best_in_range(current_offset, size);
            self.last_actions[cat_idx] = current_offset + best_action_in_cat;
            results.push(best_action_in_cat as i32);
            current_offset += size;
        }

        results
    }

    fn get_best_in_range(&self, offset: usize, size: usize) -> usize {
        let mwso_scores = self.mwso.get_action_scores(offset, size);
        let mut best = 0;
        let mut max_score = -f32::INFINITY;

        for i in 0..size {
            let wave_score = mwso_scores[i];
            
            let neuron_boost = match i {
                0 => self.nodes[self.idx_aggression].state * 0.5,
                1 => self.nodes[self.idx_fear].state * 0.3,
                _ => 0.0,
            };
            
            let fatigue_penalty = self.fatigue_map[offset + i] * 0.5;
            let score = wave_score + neuron_boost + (self.morale * 0.1) - fatigue_penalty;

            if score > max_score {
                max_score = score;
                best = i;
            }
        }
        best
    }

    pub fn learn(&mut self, reward: f32) {
        self.mwso.adapt(reward, &self.last_actions);

        let pseudo_td_error = reward.abs();
        self.digest_experience(
            pseudo_td_error,
            reward,
            if reward < 0.0 { reward.abs() } else { 0.0 },
        );
    }

    pub fn digest_experience(&mut self, td_error: f32, reward: f32, penalty: f32) {
        self.system_temperature += td_error.abs() * 0.25;
        if reward > 0.0 {
            self.system_temperature -= reward * 0.45;
        }
        self.system_temperature = (self.system_temperature.clamp(0.0, 2.0)) * 0.94;

        let urgency = ((reward + penalty) * 5.0).min(1.0);

        let mut feedback_wave = vec![0.0; 256];
        feedback_wave[0] = reward;
        feedback_wave[1] = penalty;
        self.mwso.step(&feedback_wave, 0.05);

        let current_states: Vec<f32> = self.nodes.iter().map(|n| n.state).collect();
        for node in &mut self.nodes {
            node.update(0.0, urgency, self.system_temperature, &current_states);
        }

        let temp_diff = (self.system_temperature - self.last_topology_update_temp).abs();
        if urgency > 0.5 || temp_diff > 0.05 {
            self.reshape_topology();
        }

        let all_indices: Vec<usize> = (0..self.nodes.len()).collect();
        self.horizon
            .regulate(self.system_temperature, &all_indices, &mut self.nodes);
    }

    pub fn reshape_topology(&mut self) {
        self.last_topology_update_temp = self.system_temperature;
        for node in &mut self.nodes {
            node.synapses.clear();
        }

        let glia_intervention = self.horizon.get_intervention_level();

        if self.system_temperature > 1.2 {
            self.connect(self.idx_aggression, self.idx_reflex, 2.0);
            if glia_intervention > 0.6 {
                self.nodes[self.idx_aggression].synapses.clear();
                self.connect(self.idx_reflex, self.idx_fear, 1.5);
                self.connect(self.idx_fear, self.idx_reflex, 1.2);
            }
        } else if self.system_temperature < 0.3 {
            if self.fatigue_map[0] < 0.5 {
                self.connect(self.idx_tactical, self.idx_aggression, 1.2);
            }
            self.connect(self.idx_tactical, self.idx_reflex, 0.5);
        } else {
            self.connect(self.idx_tactical, self.idx_reflex, 1.0);
            if self.adrenaline > 0.5 {
                self.connect(self.idx_aggression, self.idx_reflex, 1.5);
            } else {
                self.connect(self.idx_fear, self.idx_reflex, 0.8);
                self.connect(self.idx_aggression, self.idx_tactical, 0.7);
            }
            if self.frustration > 0.7 {
                self.connect(self.idx_tactical, self.idx_aggression, 1.8);
            }
        }
        self.apply_elastic_fatigue();
    }

    fn apply_elastic_fatigue(&mut self) {
        for (idx, fatigue) in self.fatigue_map.iter().enumerate() {
            if *fatigue > 0.8 {
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
        self.mwso.step(input_signals, 0.1);
        let current_states: Vec<f32> = self.nodes.iter().map(|n| n.state).collect();
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let input = input_signals.get(i).cloned().unwrap_or(0.0);
            node.update(input, urgency, self.system_temperature, &current_states);
        }
    }

    fn connect(&mut self, from: usize, to: usize, weight: f32) {
        self.nodes[from].synapses.push(super::node::Synapse {
            target_id: to,
            weight,
        });
    }

    pub fn set_neuron_state(&mut self, idx: usize, state: f32) {
        if let Some(node) = self.nodes.get_mut(idx) {
            node.state = state.clamp(0.0, 1.0);
        }
    }

    pub fn generate_visual_snapshot(&self, path: &str) -> bool {
        super::visualizer::Visualizer::render_wave_snapshot(&self.mwso, path).is_ok()
    }

    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(b"DSYM")?;
        file.write_all(&6u32.to_le_bytes())?; // Version 6 (No Q-Table)

        file.write_all(&(self.state_size as u32).to_le_bytes())?;
        file.write_all(&self.system_temperature.to_le_bytes())?;
        file.write_all(&self.adrenaline.to_le_bytes())?;
        file.write_all(&self.frustration.to_le_bytes())?;
        file.write_all(&self.velocity_trust.to_le_bytes())?;
        file.write_all(&self.morale.to_le_bytes())?;
        file.write_all(&self.patience.to_le_bytes())?;
        file.write_all(&self.exploration_beta.to_le_bytes())?;
        file.write_all(&self.horizon.glutamate_buffer.to_le_bytes())?;

        for f in &self.fatigue_map {
            file.write_all(&f.to_le_bytes())?;
        }

        file.write_all(&(self.category_sizes.len() as u32).to_le_bytes())?;
        for &s in &self.category_sizes {
            file.write_all(&(s as u32).to_le_bytes())?;
        }

        file.write_all(&(self.nodes.len() as u32).to_le_bytes())?;
        for node in &self.nodes {
            file.write_all(&node.state.to_le_bytes())?;
            file.write_all(&node.base_decay.to_le_bytes())?;
        }

        for &f in &self.mwso.psi_real { file.write_all(&f.to_le_bytes())?; }
        for &f in &self.mwso.psi_imag { file.write_all(&f.to_le_bytes())?; }
        for &f in &self.mwso.theta { file.write_all(&f.to_le_bytes())?; }

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
        if version >= 4 {
            let loaded_state_size = read_u32(&mut cursor)? as usize;
            if loaded_state_size != self.state_size {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Incompatible state size"));
            }
        }

        self.system_temperature = read_f32(&mut cursor)?;
        self.adrenaline = read_f32(&mut cursor)?;
        self.frustration = read_f32(&mut cursor)?;
        self.velocity_trust = read_f32(&mut cursor)?;
        self.morale = read_f32(&mut cursor)?;
        self.patience = read_f32(&mut cursor)?;
        if version >= 3 { self.exploration_beta = read_f32(&mut cursor)?; }
        if version >= 4 { self.horizon.glutamate_buffer = read_f32(&mut cursor)?; }

        if version >= 2 {
            for i in 0..self.action_size { self.fatigue_map[i] = read_f32(&mut cursor)?; }
            if version < 6 {
                if version >= 3 { cursor += self.state_size * self.action_size * 4; } // Skip visit_counts
            }
            let cat_count = read_u32(&mut cursor)? as usize;
            let mut loaded_cats = Vec::with_capacity(cat_count);
            for _ in 0..cat_count { loaded_cats.push(read_u32(&mut cursor)? as usize); }
            if version < 6 {
                cursor += (self.state_size * loaded_cats.iter().sum::<usize>()) * 4; // Skip Q-Table
            }
        }

        let node_count = read_u32(&mut cursor)? as usize;
        for i in 0..node_count.min(self.nodes.len()) {
            self.nodes[i].state = read_f32(&mut cursor)?;
            self.nodes[i].base_decay = read_f32(&mut cursor)?;
        }

        if version >= 5 {
            for i in 0..256 { self.mwso.psi_real[i] = read_f32(&mut cursor)?; }
            for i in 0..256 { self.mwso.psi_imag[i] = read_f32(&mut cursor)?; }
            for i in 0..512 { self.mwso.theta[i] = read_f32(&mut cursor)?; }
        }

        self.last_topology_update_temp = -1.0;
        self.reshape_topology();
        Ok(())
    }
}