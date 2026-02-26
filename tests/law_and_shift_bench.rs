use dark_singularity::core::singularity::Singularity;

/// 安定した法則性への同調ベンチマーク
#[test]
fn benchmark_structured_law_sync() {
    let state_size = 20;
    let action_size = 10;
    let mut ai = Singularity::new(state_size, vec![action_size]);
    
    println!("\n--- DS-Bench: Structured Law Synchronization ---");
    println!("Task: Learn a stable linear mapping (target = state / 2)");

    let total_steps = 200;
    let mut window_correct = 0;
    let window_size = 20;

    for i in 0..total_steps {
        let state_idx = i % state_size;
        let target_action = (state_idx / 2) % action_size;

        let actions = ai.select_actions(state_idx);
        let selected = actions[0] as usize;
        let is_correct = selected == target_action;

        if is_correct {
            window_correct += 1;
        }

        let reward = if is_correct { 2.0 } else { -1.0 };
        ai.learn(reward);

        if (i + 1) % window_size == 0 {
            let acc = (window_correct as f64 / window_size as f64) * 100.0;
            println!("Steps {:03}-{:03} | Accuracy: {:>5.1}% | Temp: {:.2}", 
                i - (window_size - 1) + 1, i + 1, acc, ai.system_temperature);
            window_correct = 0;
        }
    }
}

/// 15ステップごとに法則が切り替わる「超短期適応」ベンチマーク
#[test]
fn benchmark_rapid_15step_shift() {
    let state_size = 10;
    let action_size = 8;
    let mut ai = Singularity::new(state_size, vec![action_size]);
    
    println!("\n--- DS-Bench: Rapid 15-Step Shift ---");
    println!("Task: Adaptation to a rule that shifts every 15 steps");

    let total_steps = 300; // 20回の法則変化
    let mut total_correct = 0;
    let mut current_cycle_correct = 0;
    let shift_interval = 15;

    let mut law_offset = 0;

    for i in 0..total_steps {
        // 15ステップごとに法則（オフセット）を変更
        if i > 0 && i % shift_interval == 0 {
            let prev_acc = (current_cycle_correct as f64 / shift_interval as f64) * 100.0;
            println!("Cycle {:02} | Result: {:>5.1}% | Offset: {} -> {}", 
                i / shift_interval, prev_acc, law_offset, (law_offset + 2) % action_size);
            
            law_offset = (law_offset + 2) % action_size;
            current_cycle_correct = 0;
        }

        let state_idx = i % state_size;
        let target_action = (state_idx + law_offset) % action_size;

        let actions = ai.select_actions(state_idx);
        let selected = actions[0] as usize;
        let is_correct = selected == target_action;

        if is_correct {
            total_correct += 1;
            current_cycle_correct += 1;
        }

        // 報酬設定：素早い切り替えを促すため、失敗時のペナルティを強めに設定
        let reward = if is_correct { 3.0 } else { -2.0 };
        ai.learn(reward);
    }

    let final_acc = (total_correct as f64 / total_steps as f64) * 100.0;
    println!("\nFinal Rapid-Shift Adaptability: {:.2}%", final_acc);

    // 15ステップという極短期間での平均正解率がランダム(12.5%)を上回ることを確認
    assert!(final_acc > 20.0, "AI must show agility in rapid shifting environments");
}
