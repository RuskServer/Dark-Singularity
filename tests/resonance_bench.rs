use dark_singularity::core::singularity::Singularity;

#[test]
fn benchmark_resonance_growth() {
    let mut ai = Singularity::new(10, vec![5]);
    
    println!("\n--- DS-Bench: Resonance Density Profile ---");
    
    let initial_rhyd = ai.get_resonance_density();
    println!("Initial State: {:.4} Rhyd", initial_rhyd);

    // 10ステップの「共鳴学習」を実行
    for i in 1..=10 {
        ai.select_actions(0);
        ai.learn(1.0); // 常に正の報酬を与える
        let current_rhyd = ai.get_resonance_density();
        println!("Step {:02}: {:.4} Rhyd", i, current_rhyd);
    }

    let final_rhyd = ai.get_resonance_density();
    assert!(final_rhyd > initial_rhyd, "Intelligence (Rhyd) should grow with learning");
    println!("Total Intelligence Growth: {:.2}%\n", (final_rhyd / initial_rhyd - 1.0) * 100.0);
}

#[test]
fn benchmark_liquid_memory_retention() {
    let mut ai = Singularity::new(10, vec![5]);
    
    // 十分に学習させる
    for _ in 0..20 {
        ai.select_actions(0);
        ai.learn(1.0);
    }
    
    let learned_rhyd = ai.get_resonance_density();
    println!("Learned Resonance: {:.4} Rhyd", learned_rhyd);

    // 入力を止め、時間だけを進める (Reverb test)
    println!("Testing Liquid Memory Retention...");
    for i in 1..=5 {
        ai.update_all_nodes(&[0.0; 4], 0.1); // 無入力
        let current_rhyd = ai.get_resonance_density();
        println!("Persistence T+{:01}: {:.4} Rhyd", i, current_rhyd);
    }
}
