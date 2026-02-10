use dark_singularity::core::singularity::Singularity;

/// カオス適応力ベンチマーク
/// ロジスティック写像による予測困難な状態遷移と、動的に変化する正解ルールへの追従性を測定する
#[test]
fn benchmark_chaos_dynamic_adaptation() {
    let state_size = 100; // 広大な状態空間
    let action_size = 20;  // 選択肢の多さ
    let mut ai = Singularity::new(state_size, vec![action_size]);
    
    println!("\n--- DS-Bench: Chaos & Dynamic Adaptation ---");
    println!("Task: Follow shifting targets in a logistic-map driven environment");

    let mut x = 0.7; // ロジスティック写像の初期値
    let r = 3.99;    // カオス領域のパラメータ

    let mut correct_count = 0;
    let mut window_correct = 0;
    let window_size = 200;
    let total_steps = 2000;
    
    // 1000ステップごとに「世界の法則（正解アクションのオフセット）」が変わる
    let mut law_offset = 0;

    for i in 0..total_steps {
        // 1. カオス写像による状態の決定
        x = r * x * (1.0 - x);
        let state_idx = (x * (state_size as f32 - 1.0)) as usize;

        // 2. 動的な法則の適用
        if i == 1000 {
            println!("\n[EVENT] World Law Shifted! Re-adaptation required.");
            law_offset = 7; // 法則が変化
            window_correct = 0;
        }

        // 期待されるアクション（非線形な対応関係）
        let target_action = (state_idx + law_offset) % action_size;

        // 3. 意志決定
        let actions = ai.select_actions(state_idx);
        let selected = actions[0] as usize;
        let is_correct = selected == target_action;

        if is_correct {
            correct_count += 1;
            window_correct += 1;
        }

        // 4. 学習（カオス環境下での報酬フィードバック）
        // 学習履歴（Time-series learning）がここで活きる
        let reward = if is_correct { 3.0 } else { -1.5 };
        ai.learn(reward);

        // 5. 進捗表示
        if (i + 1) % window_size == 0 {
            let acc = (window_correct as f32 / window_size as f32) * 100.0;
            println!("Steps {:04}-{:04} | Law Offset: {} | Window Accuracy: {:>5.1}% | Rhyd: {:.2}", 
                i - (window_size - 1) + 1, i + 1, law_offset, acc, ai.get_resonance_density());
            window_correct = 0;
        }
    }

    let total_acc = (correct_count as f32 / total_steps as f32) * 100.0;
    println!("Final Chaos-Adaptation Score: {:.2}%", total_acc);

    // カオスかつ動的環境下でも、ランダム(5%)を上回る適応力を期待
    assert!(total_acc > 10.0, "AI should demonstrate adaptability in chaotic environments");
}

/// 複雑な干渉波の中での特定パターン抽出能力を測る
#[test]
fn benchmark_complex_interference_resolution() {
    let mut ai = Singularity::new(50, vec![10]);
    
    println!("\n--- DS-Bench: Complex Interference Resolution ---");
    println!("Task: Extract signal from simultaneous multi-state inputs");

    let total_steps = 300;
    let mut correct = 0;

    for i in 0..total_steps {
        // 3つの状態が「重畳」している状況をシミュレート
        // 実際には1つずつ注入されるが、直近の履歴として残る
        let s1 = (i % 5) as usize;
        let s2 = (i % 7 + 10) as usize;
        let s3 = (i % 11 + 20) as usize;

        // 目標: 最も支配的な s1 に対応するアクションを選択すること
        let target = (s1 * 2) % 10;

        ai.select_actions(s3);
        ai.select_actions(s2);
        let actions = ai.select_actions(s1); // s1 が最新（最も強いはず）
        
        let selected = actions[0] as usize;
        if selected == target {
            correct += 1;
        }

        let reward = if selected == target { 2.0 } else { -0.5 };
        ai.learn(reward);

        if (i + 1) % 100 == 0 {
            println!("Step {:03} | Interference Resolution Accuracy: {:.1}%", i + 1, (correct as f32 / (i + 1) as f32) * 100.0);
        }
    }

    assert!(correct as f32 / total_steps as f32 > 0.4, "Should resolve primary signal among interference");
}
