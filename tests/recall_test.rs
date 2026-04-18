use dark_singularity::core::mwso::MWSO;

#[test]
fn test_memory_recall_quality() {
    let mut mwso = MWSO::new(1024);
    let state_idx = 42;
    
    // 1. Set a specific pattern in psi
    for i in 0..1024 {
        if i % 100 == 0 {
            mwso.psi_real[i] = 1.0;
            mwso.psi_imag[i] = 0.5;
        } else {
            mwso.psi_real[i] = 0.0;
            mwso.psi_imag[i] = 0.0;
        }
    }
    
    println!("Imprinting state {} with reward 1.0", state_idx);
    mwso.imprint_qcel(state_idx, 1.0);
    
    // 2. Clear psi
    for i in 0..1024 {
        mwso.psi_real[i] = 0.0;
        mwso.psi_imag[i] = 0.0;
    }
    
    // 3. Set query
    mwso.set_input_query(state_idx, 1.0);
    
    // 4. Inspect what recall_re/im would be in step_core
    let mut recall_re = vec![0.0; mwso.dim];
    let mut recall_im = vec![0.0; mwso.dim];
    
    let system_temp: f64 = 0.0;
    // Fix: use f64 for clamp
    let gate_power = (2.2f64 - system_temp * 1.0f64).clamp(1.2, 3.5);

    for i in 0..mwso.dim {
        let sig_re = mwso.input_signature_re[i] as f64;
        // Suspicious noise line from mwso.rs
        let sig_im = (mwso.scramble_phases[i] + mwso.rng_seed as f32).sin() as f64 * 0.2;

        let sig_mag = (sig_re.powi(2) + sig_im.powi(2)).sqrt() + 1e-9;
        let u_sig_re = sig_re / sig_mag;
        let u_sig_im = sig_im / sig_mag;

        let rec_re = mwso.q_memory_re[i] * u_sig_re - mwso.q_memory_im[i] * u_sig_im;
        let rec_im = mwso.q_memory_re[i] * u_sig_im + mwso.q_memory_im[i] * u_sig_re;

        recall_re[i] = rec_re as f32;
        recall_im[i] = rec_im as f32;
    }
    
    let mut energy = 0.0f32;
    for i in 0..1024 {
        energy += recall_re[i].powi(2) + recall_im[i].powi(2);
    }
    
    println!("Recall energy: {}", energy);
    
    // Check if peaks at 0, 100, 200... are restored
    for i in (0..1024).step_by(100) {
        let mag = (recall_re[i].powi(2) + recall_im[i].powi(2)).sqrt();
        println!("Index {}: Mag={:.4}, Real={:.4}, Imag={:.4}", i, mag, recall_re[i], recall_im[i]);
        // Expect magnitude to be significantly higher than 0
        assert!(mag > 0.001, "Recall at index {} is too weak: {}", i, mag);
    }
}

use dark_singularity::core::singularity::Singularity;

#[test]
fn test_vector_state_hole_filling() {
    // 20 states, 16 actions
    let mut singularity = Singularity::new(20, vec![16]);
    
    // 1. Learn association: (State 5 + State 10) -> Action 7
    let combined_state = vec![(5, 1.0), (10, 1.0)];
    let target_action = 7;
    
    println!("\nLearning: (S5 + S10) -> A7");
    for _ in 0..100 {
        let actions = singularity.select_actions_vector(&combined_state);
        let reward = if actions[0] == target_action as i32 { 1.5 } else { -0.5 };
        // Force the correct action learning if not selected (teaching mode)
        if actions[0] != target_action as i32 {
            singularity.observe_expert(5, &vec![target_action], 0.8);
            singularity.observe_expert(10, &vec![target_action], 0.8);
        }
        singularity.learn_vector(reward);
    }
    
    // 2. Test Hole-Filling: Provide ONLY State 5, see if it recalls Action 7
    println!("Testing Hole-Filling: S5 only -> ?");
    let partial_state = vec![(5, 1.0)];
    
    // Increase focus for better recall
    singularity.adrenaline = 1.0; 
    
    let mut hits = 0;
    for _ in 0..10 {
        // Let it settle for a few steps with the query
        for _ in 0..3 {
            singularity.select_actions_vector(&partial_state);
        }
        let actions = singularity.select_actions_vector(&partial_state);
        println!("  Selected Action: {}", actions[0]);
        if actions[0] == target_action as i32 {
            hits += 1;
        }
    }
    
    assert!(hits >= 6, "Hole-filling failed! Expected Action {}, but got hits: {}/10", target_action, hits);
    println!("Hole-filling SUCCESS: Recall Action {} from partial state", target_action);
}
