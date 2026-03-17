use dark_singularity::core::singularity::Singularity;
use std::f32::consts::PI;

/// Helper to generate a distinct wave pattern for testing.
fn generate_pattern(dim: usize, seed: usize) -> (Vec<f32>, Vec<f32>) {
    let mut re = vec![0.0; dim];
    let mut im = vec![0.0; dim];
    for i in 0..dim {
        let phase = ((i + seed) as f32 * 0.1);
        re[i] = phase.cos();
        im[i] = phase.sin();
    }
    (re, im)
}

#[test]
fn benchmark_superposition_capacity() {
    let mut ai = Singularity::new(10, vec![10]);
    let dim = ai.mwso.dim;
    
    println!("\n--- DS-Bench: PP-CEL Superposition Capacity ---");
    println!("Task: Imprint multiple input-correlated patterns and measure retrieval accuracy.");

    let num_patterns = 5;
    let mut patterns = Vec::new();

    for i in 0..num_patterns {
        let (re, im) = generate_pattern(dim, i * 100);
        patterns.push((re.clone(), im.clone()));
        
        // 1. Set current wave to the pattern
        ai.mwso.psi_real = re;
        ai.mwso.psi_imag = im;
        
        // 2. Imprint into PP-CEL memory with input index i
        ai.mwso.imprint_qcel(i, 1.2);
    }

    println!("Imprinted {} patterns into correlated holographic memory.", num_patterns);

    let mut total_resonance = 0.0;
    for i in 0..num_patterns {
        // 3. Set the query for input i
        ai.mwso.set_input_query(i, 1.0);
        
        // Calculate manual resonance based on the correlation with the target pattern:
        // Recall_vector = Memory * Query
        // Resonance = DotProduct(Recall_vector, Target_Pattern) / dim
        let mut correlation = 0.0;
        let (target_re, target_im) = &patterns[i];

        for j in 0..dim {
            let sig_re = ai.mwso.input_signature[j] as f64;
            // Recall from memory using the signature
            let rec_re = ai.mwso.q_memory_re[j] * sig_re;
            let rec_im = ai.mwso.q_memory_im[j] * sig_re;
            
            // Measure how much the recall aligns with the original pattern
            correlation += (rec_re * target_re[j] as f64 + rec_im * target_im[j] as f64);
        }
        
        let resonance = (correlation as f32 / dim as f32);
        println!("Input {:02} Retrieval Resonance: {:.4}", i, resonance);
        total_resonance += resonance;
    }

    let avg_resonance = total_resonance / num_patterns as f32;
    println!("Average Retrieval Resonance: {:.4}", avg_resonance);

    // In high dimensions (e.g. 1024), the per-element energy is diluted by 1/dim.
    // The threshold should be dimension-aware. 
    // Expected resonance is ~ (Memory_Norm * Query_Norm * 0.5) / dim = (5.0 * 1.0 * 0.5) / 1024 ≈ 0.0024
    let threshold = 2.0 / dim as f32; 
    assert!(avg_resonance > threshold, "Resonance should be detectable in PP-CEL memory (Avg: {:.4}, Threshold: {:.4})", avg_resonance, threshold);
}

#[test]
fn benchmark_noisy_recall_efficiency() {
    let mut ai = Singularity::new(10, vec![10]);
    let dim = ai.mwso.dim;
    
    println!("\n--- DS-Bench: PP-CEL Noisy Recall Efficiency ---");
    println!("Task: Recover a clear state from a noisy input via energy landscape traps.");

    // 1. Imprint a clear pattern for input index 0
    let (target_re, target_im) = generate_pattern(dim, 42);
    ai.mwso.psi_real = target_re.clone();
    ai.mwso.psi_imag = target_im.clone();
    ai.mwso.imprint_qcel(0, 3.0); // Strong imprint

    // 2. Create a noisy version of the target (High noise)
    // In PP-CEL, we rely on the input query to evoke the landscape.
    ai.mwso.set_input_query(0, 1.0);
    
    // Scramble current PSI to start from a random state
    for i in 0..dim {
        ai.mwso.psi_real[i] = (ai.mwso.next_rng() - 0.5);
        ai.mwso.psi_imag[i] = (ai.mwso.next_rng() - 0.5);
    }

    // Initial fidelity
    let mut initial_fidelity = 0.0;
    for j in 0..dim {
        let mag = (ai.mwso.psi_real[j].powi(2) + ai.mwso.psi_imag[j].powi(2)).sqrt() + 1e-9;
        initial_fidelity += (ai.mwso.psi_real[j]/mag * target_re[j] + ai.mwso.psi_imag[j]/mag * target_im[j]);
    }
    println!("Initial Phase Fidelity: {:.4}", initial_fidelity / dim as f32);

    // 3. Step through time and observe recovery into the "potential well"
    for step in 1..=20 {
        ai.mwso.step_core(0.1, 0.0, 1.0, 0.05, &vec![0.0; ai.mwso.dim]); // Focus, Low Temp
        
        let mut current_fidelity = 0.0;
        for j in 0..dim {
            let mag = (ai.mwso.psi_real[j].powi(2) + ai.mwso.psi_imag[j].powi(2)).sqrt() + 1e-9;
            current_fidelity += (ai.mwso.psi_real[j]/mag * target_re[j] + ai.mwso.psi_imag[j]/mag * target_im[j]);
        }
        if step % 5 == 0 {
            println!("Step {:02} Phase Fidelity: {:.4}", step, current_fidelity / dim as f32);
        }
    }
}

#[test]
fn benchmark_memory_vs_non_memory() {
    let mut ai_mem = Singularity::new(10, vec![10]);
    let mut ai_none = Singularity::new(10, vec![10]);
    let dim = ai_mem.mwso.dim;

    println!("\n--- DS-Bench: PP-CEL Memory-Driven Stability ---");
    
    let (pat_re, pat_im) = generate_pattern(dim, 77);
    
    // ai_mem has the memory for input 0
    ai_mem.mwso.psi_real = pat_re.clone();
    ai_mem.mwso.psi_imag = pat_im.clone();
    ai_mem.mwso.imprint_qcel(0, 5.0);
    ai_mem.mwso.set_input_query(0, 1.0);

    // Set both to the same starting pattern
    ai_mem.mwso.psi_real = pat_re.clone();
    ai_mem.mwso.psi_imag = pat_im.clone();
    ai_none.mwso.psi_real = pat_re.clone();
    ai_none.mwso.psi_imag = pat_im.clone();

    println!("Running 50 steps of dissipation with thermal noise...");
    for _ in 0..50 {
        ai_mem.mwso.step_core(0.1, 0.0, 0.5, 0.4, &vec![0.0; ai_mem.mwso.dim]);
        ai_none.mwso.step_core(0.1, 0.0, 0.5, 0.4, &vec![0.0; ai_none.mwso.dim]);
    }

    let mut fidelity_mem = 0.0;
    let mut fidelity_none = 0.0;
    for i in 0..dim {
        let m_mag = (ai_mem.mwso.psi_real[i].powi(2) + ai_mem.mwso.psi_imag[i].powi(2)).sqrt() + 1e-9;
        let n_mag = (ai_none.mwso.psi_real[i].powi(2) + ai_none.mwso.psi_imag[i].powi(2)).sqrt() + 1e-9;
        
        fidelity_mem += (ai_mem.mwso.psi_real[i]/m_mag * pat_re[i] + ai_mem.mwso.psi_imag[i]/m_mag * pat_im[i]);
        fidelity_none += (ai_none.mwso.psi_real[i]/n_mag * pat_re[i] + ai_none.mwso.psi_imag[i]/n_mag * pat_im[i]);
    }

    println!("Final Phase Fidelity (With Memory):    {:.4}", fidelity_mem / dim as f32);
    println!("Final Phase Fidelity (Without Memory): {:.4}", fidelity_none / dim as f32);

    assert!(fidelity_mem > fidelity_none, "PP-CEL Energy Landscape should maintain pattern fidelity!");
}
