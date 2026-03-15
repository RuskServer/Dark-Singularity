use dark_singularity::core::singularity::Singularity;
use plotters::prelude::*;
use rand::{seq::SliceRandom, thread_rng, Rng};

struct TrackingData {
    episodes: Vec<u32>,
    rewards: Vec<f32>,
    accuracies: Vec<f32>,
    iprs: Vec<f32>,
}

/// Runs a scenario where the environment rules change mid-way.
fn run_adaptation_scenario(name: &str, change_ratio: f32, action_size: usize, use_guidance_in_adaptation: bool) -> TrackingData {
    let state_size = 20;
    let category_sizes = vec![action_size];
    let mut singularity = Singularity::new(state_size, category_sizes);

    let mut data = TrackingData {
        episodes: Vec::new(),
        rewards: Vec::new(),
        accuracies: Vec::new(),
        iprs: Vec::new(),
    };

    println!("
# ADAPTATION SCENARIO: {} (Change: {}%, Dim: {}, Guidance: {})", name, change_ratio * 100.0, if singularity.sharded_mwso.is_some() { "2048" } else { "1024" }, use_guidance_in_adaptation);

    // --- Phase 1: Pre-training (always with guidance) ---
    println!("--- Phase 1: Pre-training ---");
    let mut target_map: Vec<usize> = (0..state_size).map(|i| (i * 3) % action_size).collect();
    
    let pre_train_episodes = 1500;
    let adapt_episodes = 1500;
    let max_episodes = pre_train_episodes + adapt_episodes;
    let window_size = 50;
    let mut recent_rewards = Vec::new();
    let mut recent_hits = Vec::new();

    for episode in 0..pre_train_episodes {
        let state_idx = episode % state_size;
        let correct_action = target_map[state_idx];
        let actions = singularity.select_actions(state_idx);
        let selected_action = actions[0] as usize;

        let is_correct = selected_action == correct_action;
        let reward = if is_correct { 1.2 } else { -0.8 };
        
        // Pre-training always uses guidance
        let expert_strength = if is_correct { 0.3 } else { 0.8 };
        singularity.observe_expert(state_idx, &[correct_action], expert_strength);
        singularity.learn(reward);

        recent_rewards.push(reward);
        recent_hits.push(if is_correct { 1.0 } else { 0.0 });
        if recent_rewards.len() > window_size { recent_rewards.remove(0); recent_hits.remove(0); }

        if episode % 50 == 0 && !recent_rewards.is_empty() {
            let avg_reward = recent_rewards.iter().sum::<f32>() / recent_rewards.len() as f32;
            let accuracy = recent_hits.iter().sum::<f32>() / recent_hits.len() as f32;
            data.episodes.push(episode as u32);
            data.rewards.push(avg_reward);
            data.accuracies.push(accuracy);
            data.iprs.push(singularity.calculate_current_ipr());

            if avg_reward > 1.15 && episode > 200 {
                 println!("
Pre-training converged at episode {}.", episode);
                 break;
            }
        }
    }

    // --- Phase 2: Environmental Shift ---
    println!("
--- Phase 2: Environmental Shift ---");
    let mut rng = thread_rng();
    let mut states_to_change: Vec<usize> = (0..state_size).collect();
    states_to_change.shuffle(&mut rng);
    let num_to_change = (state_size as f32 * change_ratio) as usize;

    for &state_idx in &states_to_change[..num_to_change] {
        let old_action = target_map[state_idx];
        let mut new_action;
        loop {
            new_action = rng.gen_range(0..action_size);
            if new_action != old_action { break; }
        }
        target_map[state_idx] = new_action;
        println!("Rule change: State {} -> Action {} (was {})", state_idx, new_action, old_action);
    }
    
    // --- Phase 3: Adaptation ---
    println!("
--- Phase 3: Adaptation (Guidance: {}) ---", use_guidance_in_adaptation);
    recent_rewards.clear();
    recent_hits.clear();

    for episode in pre_train_episodes..max_episodes {
        let state_idx = episode % state_size;
        let correct_action = target_map[state_idx];
        let actions = singularity.select_actions(state_idx);
        let selected_action = actions[0] as usize;

        let is_correct = selected_action == correct_action;
        let reward = if is_correct { 1.2 } else { -0.8 };
        
        if use_guidance_in_adaptation {
            let expert_strength = if is_correct { 0.3 } else { 0.8 };
            singularity.observe_expert(state_idx, &[correct_action], expert_strength);
        }
        singularity.learn(reward);

        recent_rewards.push(reward);
        recent_hits.push(if is_correct { 1.0 } else { 0.0 });
        if recent_rewards.len() > window_size { recent_rewards.remove(0); recent_hits.remove(0); }

        if episode % 50 == 0 && !recent_rewards.is_empty() {
            let avg_reward = recent_rewards.iter().sum::<f32>() / recent_rewards.len() as f32;
            let accuracy = recent_hits.iter().sum::<f32>() / recent_hits.len() as f32;
            data.episodes.push(episode as u32);
            data.rewards.push(avg_reward);
            data.accuracies.push(accuracy);
            data.iprs.push(singularity.calculate_current_ipr());
        }
    }
    println!(" Done.");
    data
}

#[test]
fn bench_adaptation_plot() -> Result<(), Box<dyn std::error::Error>> {
    let pre_train_episodes: u32 = 1500;
    let max_episodes: u32 = 3000;

    let scenarios = vec![
        ("1024-20% (Guidance)", run_adaptation_scenario("1024-20% (Guidance)", 0.2, 16, true)),
        ("1024-20% (No Guidance)", run_adaptation_scenario("1024-20% (No Guidance)", 0.2, 16, false)),
        ("2048-20% (Guidance)", run_adaptation_scenario("2048-20% (Guidance)", 0.2, 32, true)),
        ("2048-20% (No Guidance)", run_adaptation_scenario("2048-20% (No Guidance)", 0.2, 32, false)),
    ];

    let root = BitMapBackend::new("adaptation_graph.png", (1200, 1024)).into_drawing_area();
    root.fill(&WHITE)?;

    let (upper, lower) = root.split_vertically(512);

    // --- Accuracy Chart ---
    let mut chart_acc = ChartBuilder::on(&upper)
        .caption("Adaptation: Learning Accuracy", ("sans-serif", 20).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0..max_episodes, 0.0f32..1.1f32)?;

    chart_acc.configure_mesh().draw()?;

    chart_acc.draw_series(LineSeries::new(
        vec![(pre_train_episodes, 0.0), (pre_train_episodes, 1.1)],
        BLACK.mix(0.5).stroke_width(2),
    ))?
    .label("Environmental Shift")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK.mix(0.5)));

    // --- IPR Chart ---
    let mut chart_ipr = ChartBuilder::on(&lower)
        .caption("Adaptation: Wave-function Complexity (IPR)", ("sans-serif", 20).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0..max_episodes, 0.0f32..240.0f32)?;

    chart_ipr.configure_mesh().draw()?;
    
    chart_ipr.draw_series(LineSeries::new(
        vec![(pre_train_episodes, 0.0), (pre_train_episodes, 240.0)],
        BLACK.mix(0.5).stroke_width(2),
    ))?;

    let colors = [RED, BLUE, GREEN, MAGENTA];
    for (i, (name, data)) in scenarios.iter().enumerate() {
        let color = colors[i % colors.len()];
        
        chart_acc.draw_series(LineSeries::new(
            data.episodes.iter().zip(data.accuracies.iter()).map(|(&x, &y)| (x, y)),
            &color,
        ))?
        .label(*name)
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));

        chart_ipr.draw_series(LineSeries::new(
            data.episodes.iter().zip(data.iprs.iter()).map(|(&x, &y)| (x, y)),
            &color,
        ))?
        .label(*name)
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));
    }

    chart_acc.configure_series_labels().background_style(&WHITE.mix(0.8)).border_style(&BLACK).draw()?;
    chart_ipr.configure_series_labels().background_style(&WHITE.mix(0.8)).border_style(&BLACK).draw()?;

    root.present()?;
    println!("Adaptation graph saved to adaptation_graph.png");
    Ok(())
}
