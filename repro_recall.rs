use dark_singularity::core::mwso::MWSO;

fn main() {
    let mut mwso = MWSO::new(1024);
    let state_idx = 42;
    
    // 1. Give some reward to current state (which is mostly noise/0.01)
    // First, let's set a specific pattern in psi
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
    
    // 4. Step once to see recall in step_core (we need to bypass or inspect it)
    // Actually, let's just inspect what recall_re/im would be in step_core
    
    // Manually reproduce step_core retrieval logic
    let mut recall_re = vec![0.0; mwso.dim];
    let mut recall_im = vec![0.0; mwso.dim];
    
    let system_temp = 0.0;
    let gate_power = (2.2 - system_temp * 1.0).clamp(1.2, 3.5);

    for i in 0..mwso.dim {
        let sig_re = mwso.input_signature[i] as f64;
        // This is the suspicious line in mwso.rs
        let sig_im = (mwso.scramble_phases[i] + mwso.rng_seed as f32).sin() as f64 * 0.2;

        let sig_mag = (sig_re.powi(2) + sig_im.powi(2)).sqrt() + 1e-9;
        let u_sig_re = sig_re / sig_mag;
        let u_sig_im = sig_im / sig_mag;

        let rec_re = mwso.q_memory_re[i] * u_sig_re - mwso.q_memory_im[i] * u_sig_im;
        let rec_im = mwso.q_memory_re[i] * u_sig_im + mwso.q_memory_im[i] * u_sig_re;

        recall_re[i] = rec_re as f32;
        recall_im[i] = rec_im as f32;
    }
    
    let mut energy = 0.0;
    for i in 0..1024 {
        energy += recall_re[i].powi(2) + recall_im[i].powi(2);
    }
    
    println!("Recall energy: {}", energy);
    
    // Check if peaks at 0, 100, 200... are restored
    for i in (0..1024).step_by(100) {
        let mag = (recall_re[i].powi(2) + recall_im[i].powi(2)).sqrt();
        println!("Index {}: Mag={:.4}, Real={:.4}, Imag={:.4}", i, mag, recall_re[i], recall_im[i]);
    }
}
