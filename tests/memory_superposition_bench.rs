use dark_singularity::core::singularity::Singularity;

/// Helper to generate a distinct wave pattern for testing.
fn generate_pattern(dim: usize, seed: usize) -> (Vec<f32>, Vec<f32>) {
    let mut re = vec![0.0; dim];
    let mut im = vec![0.0; dim];
    for i in 0..dim {
        let val = ((i + seed) as f32 * 0.1).sin();
        re[i] = val;
        im[i] = ((i + seed) as f32 * 0.1).cos();
    }
    (re, im)
}

#[test]
fn benchmark_superposition_capacity() {
    let mut ai = Singularity::new(10, vec![10]);
    let dim = ai.mwso.dim;
    
    println!("
--- DS-Bench: Quantum Superposition Capacity ---");
    println!("Task: Imprint multiple distinct patterns and measure resonance accuracy.");

    let num_patterns = 5;
    let mut patterns = Vec::new();

    for i in 0..num_patterns {
        let (re, im) = generate_pattern(dim, i * 100);
        patterns.push((re.clone(), im.clone()));
        // Imprint into global memory wave
        ai.mwso.imprint_memory(&re, &im, 1.0);
    }

    println!("Imprinted {} patterns into a single memory wave.", num_patterns);

    let mut total_resonance = 0.0;
    for i in 0..num_patterns {
        // Set current state to pattern i
        ai.mwso.psi_real = patterns[i].0.clone();
        ai.mwso.psi_imag = patterns[i].1.clone();

        // Calculate resonance amplitude (how well it matches memory)
        // We simulate the resonance logic from step_core
        let mut overlap_re = 0.0;
        let mut overlap_im = 0.0;
        for j in 0..dim {
            overlap_re += ai.mwso.psi_real[j] * ai.mwso.memory_psi_real[j] + ai.mwso.psi_imag[j] * ai.mwso.memory_psi_imag[j];
            overlap_im += ai.mwso.psi_real[j] * ai.mwso.memory_psi_imag[j] - ai.mwso.psi_imag[j] * ai.mwso.memory_psi_real[j];
        }
        let resonance = (overlap_re.powi(2) + overlap_im.powi(2)).sqrt();
        println!("Pattern {:02} Resonance: {:.4}", i, resonance);
        total_resonance += resonance;
    }

    let avg_resonance = total_resonance / num_patterns as f32;
    println!("Average Retrieval Resonance: {:.4}", avg_resonance);

    // Each pattern should have a high resonance. 
    // Since they are superimposed and normalized, the overlap with 'self' in the memory 
    // should be significantly higher than with random noise.
    assert!(avg_resonance > 0.3, "Resonance should be detectable even in superposition.");
}

#[test]
fn benchmark_noisy_recall_efficiency() {
    let mut ai = Singularity::new(10, vec![10]);
    let dim = ai.mwso.dim;
    
    println!("
--- DS-Bench: Noisy Memory Recall Efficiency ---");
    println!("Task: Recover a clear state from a noisy input via memory resonance.");

    // 1. Imprint a clear pattern
    let (target_re, target_im) = generate_pattern(dim, 42);
    ai.mwso.imprint_memory(&target_re, &target_im, 2.0);

    // 2. Create a noisy version of the target (50% noise)
    let mut noisy_re = target_re.clone();
    let mut noisy_im = target_im.clone();
    for i in 0..dim {
        if i % 2 == 0 {
            noisy_re[i] = 0.0;
            noisy_im[i] = 0.0;
        }
    }
    
    ai.mwso.psi_real = noisy_re.clone();
    ai.mwso.psi_imag = noisy_im.clone();

    // Initial fidelity
    let mut dot = 0.0;
    for j in 0..dim {
        dot += ai.mwso.psi_real[j] * target_re[j] + ai.mwso.psi_imag[j] * target_im[j];
    }
    println!("Initial Fidelity: {:.4}", dot);

    // 3. Step through time and observe recovery
    for step in 1..=10 {
        ai.mwso.step_core(0.1, 0.0, 1.0, 0.1); // High focus, low temp for recall
        
        let mut current_dot = 0.0;
        for j in 0..dim {
            current_dot += ai.mwso.psi_real[j] * target_re[j] + ai.mwso.psi_imag[j] * target_im[j];
        }
        println!("Step {:02} Fidelity: {:.4}", step, current_dot);
    }
}

#[test]
fn benchmark_memory_vs_non_memory() {
    let mut ai_mem = Singularity::new(10, vec![10]);
    let mut ai_none = Singularity::new(10, vec![10]);
    let dim = ai_mem.mwso.dim;

    println!("
--- DS-Bench: Memory-Driven Stability ---");
    
    let (pat_re, pat_im) = generate_pattern(dim, 77);
    
    // ai_mem has the memory, ai_none does not.
    ai_mem.mwso.imprint_memory(&pat_re, &pat_im, 5.0);

    // Set both to the pattern
    ai_mem.mwso.psi_real = pat_re.clone();
    ai_mem.mwso.psi_imag = pat_im.clone();
    ai_none.mwso.psi_real = pat_re.clone();
    ai_none.mwso.psi_imag = pat_im.clone();

    println!("Running 50 steps of dissipation...");
    for _ in 0..50 {
        ai_mem.mwso.step_core(0.1, 0.0, 0.0, 0.5);
        ai_none.mwso.step_core(0.1, 0.0, 0.0, 0.5);
    }

    // Measure fidelity (overlap) with the original pattern
    let mut fidelity_mem = 0.0;
    let mut fidelity_none = 0.0;
    for i in 0..dim {
        fidelity_mem += ai_mem.mwso.psi_real[i] * pat_re[i] + ai_mem.mwso.psi_imag[i] * pat_im[i];
        fidelity_none += ai_none.mwso.psi_real[i] * pat_re[i] + ai_none.mwso.psi_imag[i] * pat_im[i];
    }

    println!("Final Fidelity (With Memory):    {:.4}", fidelity_mem);
    println!("Final Fidelity (Without Memory): {:.4}", fidelity_none);

    assert!(fidelity_mem > fidelity_none, "Memory resonance should maintain pattern fidelity!");
    println!("Fidelity Improvement: {:.1}%", (fidelity_mem / fidelity_none - 1.0) * 100.0);
}
