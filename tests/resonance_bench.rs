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

#[test]
fn benchmark_future_prediction_accuracy() {
    let mut ai = Singularity::new(10, vec![10]); // 1カテゴリー, 10アクション
    
    println!("\n--- DS-Bench: Future Prediction Accuracy ---");
    println!("Task: Cyclic state-action mapping (3 states, 10 actions)");
    
    let mut correct_count = 0;
    let mut window_correct = 0;
    let window_size = 50;
    let total_steps = 250;
    
    for i in 0..total_steps {
        let state_idx = i % 3;
        let target_action = (state_idx * 3 + 1) % 10; // シャッフル
        
        let actions = ai.select_actions(state_idx);
        let selected = actions[0] as usize;
        
        let is_correct = selected == target_action;
        if is_correct {
            correct_count += 1;
            window_correct += 1;
        }
        
        // 学習フェーズ (位相の同調)
        let reward = if is_correct { 2.0 } else { -0.5 };
        ai.learn(reward);
        
        if (i + 1) % window_size == 0 {
            let acc = (window_correct as f32 / window_size as f32) * 100.0;
            println!("Steps {:03}-{:03} | Success Rate: {:>5.1}%", i - (window_size - 1) + 1, i + 1, acc);
            window_correct = 0;
        }
    }
    
    let total_acc = (correct_count as f32 / total_steps as f32) * 100.0;
    println!("Global Predictive Accuracy: {:.2}%", total_acc);
    
    // ランダム(10%)よりは有意に高いはず
    assert!(total_acc > 20.0, "Prediction accuracy should exceed random chance");
}

#[test]
fn benchmark_knowledge_guided_prediction() {
    let mut ai = Singularity::new(10, vec![10]);
    
    println!("\n--- DS-Bench: Knowledge Guided Prediction ---");
    println!("Task: State-Action mapping with Hamiltonian Guidance (Initial Knowledge)");

    // 1. 初期知識（理性）の注入
    ai.bootstrapper.add_hamiltonian_rule(100, 4, 5.0); // State 0 -> Action 4
    ai.bootstrapper.add_hamiltonian_rule(101, 7, 5.0); // State 1 -> Action 7
    ai.bootstrapper.add_hamiltonian_rule(102, 1, 5.0); // State 2 -> Action 1

    let mut correct_count = 0;
    let total_steps = 100; 
    
    for i in 0..total_steps {
        let state_idx = i % 3;
        let target_action = match state_idx {
            0 => 4,
            1 => 7,
            2 => 1,
            _ => 0,
        };

        // 2. 状況に応じた条件の発動
        match state_idx {
            0 => ai.set_active_conditions(&[100]),
            1 => ai.set_active_conditions(&[101]),
            2 => ai.set_active_conditions(&[102]),
            _ => ai.set_active_conditions(&[]),
        }

        let actions = ai.select_actions(state_idx);
        let selected = actions[0] as usize;
        
        let is_correct = selected == target_action;
        if is_correct {
            correct_count += 1;
        }

        // 学習も並行して行う（理性が本能を導く）
        let reward = if is_correct { 1.0 } else { -0.5 };
        ai.learn(reward);

        if (i + 1) % 20 == 0 {
            let acc = (correct_count as f32 / (i + 1) as f32) * 100.0;
            println!("Step {:03} | Cumulative Accuracy: {:>5.1}%", i + 1, acc);
        }
    }

    let total_acc = (correct_count as f32 / total_steps as f32) * 100.0;
    println!("Final Knowledge-Guided Accuracy: {:.2}%", total_acc);

    // 知識があるため、最初から極めて高い精度が期待できる
    assert!(total_acc > 90.0, "Knowledge guidance should yield near-perfect accuracy");
}

#[test]
fn benchmark_adaptation_to_unknown() {
    let mut ai = Singularity::new(10, vec![10]);
    ai.exploration_beta = 1.0; // 探査率を高める
    
    println!("\n--- DS-Bench: Adaptation to Unknown (Hybrid Mode) ---");
    println!("Task: State 0 (Guided) vs State 1 & 2 (Autonomous Learning)");

    // 1. State 0 のみに初期知識を注入
    ai.bootstrapper.add_hamiltonian_rule(100, 0, 5.0);

    let mut correct_known = 0;
    let mut correct_unknown = 0;
    let total_steps = 300;
    
    for i in 0..total_steps {
        let state_idx = i % 3; 
        let target_action = state_idx;

        if state_idx == 0 {
            ai.set_active_conditions(&[100]);
        } else {
            ai.set_active_conditions(&[]);
        }

        let actions = ai.select_actions(state_idx);
        let selected = actions[0] as usize;
        let is_correct = selected == target_action;

        if i % 60 < 3 {
            println!("DEBUG: Step {:03} | State {} | Selected {} | Target {} | Correct? {}", i, state_idx, selected, target_action, is_correct);
        }

        if state_idx == 0 {
            if is_correct { correct_known += 1; }
        } else {
            if is_correct { correct_unknown += 1; }
        }

        // 学習 (共通の学習メカニズム)
        let reward = if is_correct { 2.0 } else { -0.5 };
        ai.learn(reward);

        if (i + 1) % 60 == 0 {
            let steps_per_type = (i + 1) / 3;
            let acc_k = (correct_known as f32 / steps_per_type as f32) * 100.0;
            let acc_u = (correct_unknown as f32 / (steps_per_type * 2) as f32) * 100.0;
            println!("Step {:03} | Known Acc: {:>5.1}% | Unknown Acc: {:>5.1}% | Temp: {:.2}", i + 1, acc_k, acc_u, ai.system_temperature);
        }
    }

    let final_acc_k = (correct_known as f32 / (total_steps as f32 / 3.0)) * 100.0;
    let final_acc_u = (correct_unknown as f32 / (total_steps as f32 / 3.0 * 2.0)) * 100.0;
    
    println!("Final Known Stability: {:.2}%", final_acc_k);
    println!("Final Unknown Adaptability: {:.2}%", final_acc_u);

    assert!(final_acc_k > 90.0, "Should not forget pre-defined knowledge");
    assert!(final_acc_u > 20.0, "Should adapt to unknown situations even with partial knowledge");
}

#[test]
fn benchmark_high_dimensional_stress_test() {
    let num_states = 10;
    let num_actions = 16; // 512次元 / 32-bin = 16アクション
    let mut ai = Singularity::new(num_states, vec![num_actions]);
    
    println!("\n--- DS-Bench: High-Dimensional Stress Test ---");
    println!("Task: Autonomous mapping of {} states to {} actions", num_states, num_actions);

    let mut correct_count = 0;
    let mut window_correct = 0;
    let window_size = 50;
    let total_steps = 500;
    
    for i in 0..total_steps {
        let state_idx = i % num_states;
        // 真のマッピング（物理インデックスと重ならないようにシャッフル）
        // State 0 -> Act 13, State 1 -> Act 4, ... のような非自明な対応
        let target_action = (state_idx * 7 + 3) % num_actions; 

        ai.set_active_conditions(&[]);

        let actions = ai.select_actions(state_idx);
        let selected = actions[0] as usize;
        let is_correct = selected == target_action;

        if is_correct {
            correct_count += 1;
            window_correct += 1;
        }

        // 学習 (報酬フィードバック)
        let reward = if is_correct { 2.0 } else { -0.5 };
        ai.learn(reward);

        if (i + 1) % window_size == 0 {
            let acc = (window_correct as f32 / window_size as f32) * 100.0;
            println!("Steps {:03}-{:03} | Window Success Rate: {:>5.1}% | Temp: {:.2}", 
                i - (window_size - 1) + 1, i + 1, acc, ai.system_temperature);
            window_correct = 0;
        }
    }

    let total_acc = (correct_count as f32 / total_steps as f32) * 100.0;
    println!("Final Global Stress-Test Accuracy: {:.2}%", total_acc);

    // 10状態/16アクションという複雑な環境でも 70% 以上の適応を期待
    assert!(total_acc > 70.0, "High-dimensional adaptability should be robust");
}
