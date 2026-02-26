use dark_singularity::core::singularity::Singularity;

/// 逆強化学習（動機逆算）ベンチマーク
/// 報酬信号(learn)を与えず、observe_expertのみで未知の法則を習得できるか
#[test]
fn benchmark_irl_imitation_learning() {
    let state_size = 10;
    let action_size = 10;
    let mut ai = Singularity::new(state_size, vec![action_size]);
    
    println!("\n--- DS-Bench: IRL (Inverse Reinforcement Learning) ---");
    println!("Task: Acquire rule (target = state) by observing expert WITHOUT rewards");

    // 1. 観察フェーズ (観察のみ、learnは呼ばない)
    println!("Phase 1: Observing Expert...");
    for i in 0..50 {
        let state_idx = i % state_size;
        let expert_action = state_idx; // 法則: 状態と同じ番号のアクションが正解
        
        // エキスパートの行動を観測し、動機を逆算させる
        ai.observe_expert(state_idx, &[expert_action], 1.0);
        
        if (i + 1) % 10 == 0 {
            println!("  Observed {} steps...", i + 1);
        }
    }

    // 2. 自律実行フェーズ (習得した「動機」に基づいて行動できるか)
    println!("Phase 2: Autonomous Execution (Based on inferred motivation)");
    let mut correct_count = 0;
    let test_steps = 20;

    for i in 0..test_steps {
        let state_idx = i % state_size;
        let target_action = state_idx;

        // learn を呼ばず、観察で得た内部構造のみで決定
        let actions = ai.select_actions(state_idx);
        let selected = actions[0] as usize;
        
        if selected == target_action {
            correct_count += 1;
        }
        
        println!("  State {} | AI selected {} | Target {} | {}", 
            state_idx, selected, target_action, if selected == target_action { "MATCH" } else { "MISS" });
    }

    let acc = (correct_count as f64 / test_steps as f64) * 100.0;
    println!("Final IRL Acquisition Accuracy: {:.2}%", acc);

    // 報酬なしの観察のみで、ランダム(10%)を大幅に超える習得ができているはず
    assert!(acc > 50.0, "AI should infer motivation from expert actions");
}
