use dark_singularity::core::singularity::Singularity;
use std::time::Instant;

#[test]
fn benchmark_large_scale_performance() {
    let state_size = 1000;
    let cat_sizes = vec![16, 16]; // 32 actions total
    let mut ai = Singularity::new(state_size, cat_sizes);
    
    println!("\n--- DS-Perf: Large Scale Performance Test ---");
    println!("State Size: {}, Total Actions: {}", state_size, ai.action_size);

    let iterations = 100;
    
    // Measure select_actions
    let start_select = Instant::now();
    for i in 0..iterations {
        ai.select_actions(i % state_size);
    }
    let duration_select = start_select.elapsed();
    println!("select_actions (avg): {:?}", duration_select / iterations as u32);

    // Measure learn
    let start_learn = Instant::now();
    for _ in 0..iterations {
        ai.learn(1.0);
    }
    let duration_learn = start_learn.elapsed();
    println!("learn (avg): {:?}", duration_learn / iterations as u32);
    
    // Total throughput
    let total_duration = duration_select + duration_learn;
    println!("Total cycle (avg): {:?}", total_duration / iterations as u32);
    println!("Target throughput: 1000 Hz (1ms/cycle)");
    
    let avg_cycle_ms = total_duration.as_secs_f32() * 1000.0 / iterations as f32;
    if avg_cycle_ms > 1.0 {
        println!("WARNING: Performance below target! {:.2} ms/cycle", avg_cycle_ms);
    } else {
        println!("SUCCESS: Performance within target. {:.2} ms/cycle", avg_cycle_ms);
    }
}
