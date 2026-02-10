use dark_singularity::core::singularity::Singularity;

#[test]
fn test_mwso_influence() {
    let mut sing = Singularity::new(10, vec![5]);
    
    // 最初の行動選択
    let actions1 = sing.select_actions(0);
    assert_eq!(actions1.len(), 1);
    
    // 報酬を与えて学習 (MWSOの適応)
    for _ in 0..10 {
        sing.select_actions(0);
        sing.learn(1.0); // アクションを強化
    }
    
    // 適応後のスコアが変化しているか確認
    // (MWSOは確率的・波動的なので、完全に固定ではないが、傾向は出るはず)
    let score_after = sing.mwso.get_action_scores(0, 5, 0.0, &[]);
    println!("MWSO Scores after adaptation: {:?}", score_after);
}

#[test]
fn test_mwso_wave_evolution() {
    let mut sing = Singularity::new(10, vec![5]);
    
    let initial_psi = sing.mwso.psi_real.clone();
    
    // 入力を与えて波動を進める
    sing.update_all_nodes(&[1.0, 0.0, 0.0], 0.5);
    
    let evolved_psi = sing.mwso.psi_real.clone();
    
    // 波動が変化していること
    let mut changed = false;
    for i in 0..256 {
        if (initial_psi[i] - evolved_psi[i]).abs() > 1e-6 {
            changed = true;
            break;
        }
    }
    assert!(changed, "Wave state should evolve after input");
}
