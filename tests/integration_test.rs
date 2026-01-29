use dark_singularity::core::singularity::Singularity;
use std::fs;

#[test]
fn test_save_and_load() {
    let mut sing = Singularity::new();

    // Modify state
    sing.system_temperature = 0.8;
    sing.adrenaline = 0.5;
    sing.q_table[0] = 1.23;
    sing.q_table[100] = 4.56;
    sing.fatigue_map[0] = 0.9;

    let path = "test_brain.dsym";

    // Save
    sing.save_to_file(path).expect("Failed to save");

    // Create new instance
    let mut loaded_sing = Singularity::new();

    // Init state should be different
    assert_ne!(loaded_sing.system_temperature, 0.8);

    // Load
    loaded_sing.load_from_file(path).expect("Failed to load");

    // Verify
    assert!((loaded_sing.system_temperature - 0.8).abs() < 1e-6);
    assert!((loaded_sing.adrenaline - 0.5).abs() < 1e-6);
    assert!((loaded_sing.q_table[0] - 1.23).abs() < 1e-6);
    assert!((loaded_sing.q_table[100] - 4.56).abs() < 1e-6);
    assert!((loaded_sing.fatigue_map[0] - 0.9).abs() < 1e-6);

    // Cleanup
    let _ = fs::remove_file(path);
}

#[test]
fn test_optimization_logic() {
    let mut sing = Singularity::new();
    sing.system_temperature = 0.5;
    sing.last_topology_update_temp = 0.5; // Sync

    // Small change - should NOT trigger topology reshape (update temp should remain same)
    // We can't easily mock reshape_topology to spy on it, but we can check the side effect:
    // reshape_topology updates last_topology_update_temp.

    // 1. urgency=0.0, reward=0.01 (small temp change)
    // digest_experience will modify system_temperature slightly.
    // if change < 0.05, reshape shouldn't happen -> last_topology_update_temp should NOT change?
    // Wait, last_topology_update_temp represents the temp AT THE MOMENT of update.
    // If update happens, last_topology_update_temp becomes current system_temperature.
    // If update skipped, last_topology_update_temp remains old value.

    // Force known state
    sing.system_temperature = 0.50;
    sing.last_topology_update_temp = 0.50;

    // Call digest with very small error/reward -> small temp change
    // reward=0.0, error=0.04 (causes temp += 0.01) -> temp becomes ~0.51
    // Diff 0.01 < 0.05 -> Skip reshape -> last_temp remains 0.50
    sing.digest_experience(0.04, 0.0, 0.0);

    // assert!(sing.system_temperature > 0.50); // Incorrect assumption due to cooling factor
    // Instead just verify correct skip behavior:
    assert_eq!(
        sing.last_topology_update_temp, 0.50,
        "Should skip reshape for small diff"
    );

    // Large change
    // error=1.0 -> temp increase ~0.25 -> temp ~0.76
    // Diff > 0.05 -> Do reshape -> last_temp becomes current temp
    sing.digest_experience(1.0, 0.0, 0.0);

    assert!(
        (sing.last_topology_update_temp - sing.system_temperature).abs() < 1e-6,
        "Should reshape for large diff"
    );
}
