use dark_singularity::core::singularity::Singularity;
use std::fs;

#[test]
fn test_save_and_load() {
    let state_size = 64;
    let cat_sizes = vec![8];
    let mut sing = Singularity::new(state_size, cat_sizes.clone());

    // Modify state
    sing.system_temperature = 0.8;
    sing.adrenaline = 0.5;
    sing.mwso.theta[0] = 1.23;
    sing.mwso.theta[511] = 0.99;
    sing.fatigue_map[0] = 0.9;

    let path = "test_brain_v6.dsym";

    // Save
    sing.save_to_file(path).expect("Failed to save");

    // Create new instance
    let mut loaded_sing = Singularity::new(state_size, cat_sizes);

    // Init state should be different
    assert_ne!(loaded_sing.system_temperature, 0.8);

    // Load
    loaded_sing.load_from_file(path).expect("Failed to load");

    // Verify
    assert!((loaded_sing.system_temperature - 0.8).abs() < 1e-6);
    assert!((loaded_sing.adrenaline - 0.5).abs() < 1e-6);
    assert!((loaded_sing.mwso.theta[0] - 1.23).abs() < 1e-6);
    assert!((loaded_sing.mwso.theta[511] - 0.99).abs() < 1e-6);
    assert!((loaded_sing.fatigue_map[0] - 0.9).abs() < 1e-6);

    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_state_size_mismatch_validation() {
    let path = "mismatch_test_v6.dsym";
    
    // 1. 状態数 64 で保存
    {
        let sing = Singularity::new(64, vec![4]);
        sing.save_to_file(path).expect("Save failed");
    }

    // 2. 状態数 128 のインスタンスでロードを試みる -> エラーになるべき
    {
        let mut sing = Singularity::new(128, vec![4]);
        let result = sing.load_from_file(path);
        assert!(result.is_err(), "Should fail due to state_size mismatch");
    }

    let _ = fs::remove_file(path);
}

#[test]
fn test_optimization_logic() {
    let mut sing = Singularity::new(64, vec![8]);
    sing.system_temperature = 0.5;
    sing.last_topology_update_temp = 0.5; // Sync

    // Small change - should NOT trigger topology reshape
    sing.digest_experience(0.04, 0.0, 0.0);

    assert_eq!(
        sing.last_topology_update_temp, 0.50,
        "Should skip reshape for small diff"
    );

    // Large change
    sing.digest_experience(1.0, 0.0, 0.0);

    assert!(
        (sing.last_topology_update_temp - sing.system_temperature).abs() < 1e-6,
        "Should reshape for large diff"
    );
}

#[test]
fn test_knowledge_bootstrap() {
    use dark_singularity::core::knowledge::Bootstrapper;

    let state_size = 10;
    let cat_sizes = vec![4];
    let mut sing = Singularity::new(state_size, cat_sizes);

    // Save initial theta value
    let initial_theta = sing.mwso.theta[16]; // action 1 start index

    let mut bootstrapper = Bootstrapper::new();
    bootstrapper.add_rule(1, 1, 0.95); // action 1 -> bias 0.95

    bootstrapper.apply(&mut sing);

    // Theta should be updated
    assert!(sing.mwso.theta[16] > initial_theta, "Theta should increase by bootstrap");
    
    // Verify it affects action selection
    // Note: MWSO is dynamic, so we just ensure it selects *something* valid
    let actions = sing.select_actions(1);
    assert_eq!(actions.len(), 1);
}