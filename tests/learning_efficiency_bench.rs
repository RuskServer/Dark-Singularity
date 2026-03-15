use dark_singularity::core::singularity::Singularity;
use plotters::prelude::*;

struct TrackingData {
    episodes: Vec<i32>,
    rewards: Vec<f32>,
    accuracies: Vec<f32>,
    iprs: Vec<f32>,
}

fn run_convergence_tracking(name: &str, action_size: usize, is_rule: bool) -> TrackingData {
    let state_size = 20;
    let category_sizes = vec![action_size];
    let mut singularity = Singularity::new(state_size, category_sizes);

    let mut data = TrackingData {
        episodes: Vec::new(),
        rewards: Vec::new(),
        accuracies: Vec::new(),
        iprs: Vec::new(),
    };

    println!("\n# SCENARIO: {} (Dim: {})", name, singularity.mwso.dim);

    let mut target_map = Vec::with_capacity(state_size);
    if is_rule {
        for i in 0..state_size { target_map.push((i * 2) % action_size); }
    } else {
        let mut seed = 42;
        for _ in 0..state_size {
            seed = (seed * 1103515245 + 12345) & 0x7fffffff;
            target_map.push(seed % action_size);
        }
    }

    let max_episodes = 3000;
    let window_size = 50;
    let mut recent_rewards = Vec::new();
    let mut recent_hits = Vec::new();

    for episode in 0..max_episodes {
        let state_idx = episode % state_size;
        let correct_action = target_map[state_idx];

        // 事前に現在のスコア分布を覗き見（デバッグ用）
        let raw_scores = singularity.get_raw_scores(action_size);

        let actions = singularity.select_actions(state_idx);
        let selected_action = actions[0] as usize;

        let is_correct = selected_action == correct_action;
        let reward = if is_correct { 1.2 } else { -0.8 };

        // 正解を直接教える（毎ステップ）
        // 不正解時は強めに、正解時は弱めに注入
        let expert_strength = if is_correct { 0.3 } else { 0.8 };
        singularity.observe_expert(state_idx, &[correct_action], expert_strength);

        singularity.learn(reward);

        recent_rewards.push(reward);
        recent_hits.push(if is_correct { 1.0 } else { 0.0 });
        if recent_rewards.len() > window_size {
            recent_rewards.remove(0);
            recent_hits.remove(0);
        }

        // --- 詳細ログの出力 ---
        if episode % 100 == 0 {
            let temp = singularity.system_temperature;
            let ipr = singularity.calculate_current_ipr();
            let rhyd = singularity.get_resonance_density();
            let correct_score = raw_scores[correct_action];
            let max_score = raw_scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

            println!("Ep: {:4} | S: {:2} | A: {:2} ({:5}) | RawS: {:>7.2} (Max:{:>7.2}) | T: {:.3} | IPR: {:>6.2} | Rhyd: {:>6.2}",
                     episode, state_idx, selected_action, is_correct,
                     correct_score, max_score, temp, ipr, rhyd);
        }

        if episode % 50 == 0 {
            let avg_reward = recent_rewards.iter().sum::<f32>() / recent_rewards.len() as f32;
            let accuracy = recent_hits.iter().sum::<f32>() / recent_hits.len() as f32;
            let ipr = singularity.calculate_current_ipr();
            data.episodes.push(episode as i32);
            data.rewards.push(avg_reward);
            data.accuracies.push(accuracy);
            data.iprs.push(ipr);
            print!(".");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();

            if avg_reward > 1.10 && episode > 100 {
                break;
            }
        }
    }
    println!(" Done.");
    data
}

#[test]
fn bench_convergence_plot() -> Result<(), Box<dyn std::error::Error>> {
    let results = vec![
        ("1024-Rule", run_convergence_tracking("1024-Rule", 16, true)),
        ("1024-Normal", run_convergence_tracking("1024-Normal", 16, false)),
        ("2048-Rule", run_convergence_tracking("2048-Rule", 32, true)),
        ("2048-Normal", run_convergence_tracking("2048-Normal", 32, false)),
    ];

    let root = BitMapBackend::new("convergence_graph_rust.png", (1024, 1024)).into_drawing_area();
    root.fill(&WHITE)?;

    let (upper, lower) = root.split_vertically(1024 / 2);
    let (area_reward, area_accuracy) = upper.split_vertically(1024 / 4);
    let area_ipr = lower;

    // Reward Plot
    let mut chart1 = ChartBuilder::on(&area_reward)
        .caption("Average Reward", ("sans-serif", 20).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0..3000, -1.0f32..1.5f32)?;
    chart1.configure_mesh().draw()?;

    // Accuracy Plot
    let mut chart2 = ChartBuilder::on(&area_accuracy)
        .caption("Learning Accuracy", ("sans-serif", 20).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0..3000, 0.0f32..1.1f32)?;
    chart2.configure_mesh().draw()?;

    // IPR Plot
    let mut chart3 = ChartBuilder::on(&area_ipr)
        .caption("Wave-function Complexity: IPR", ("sans-serif", 20).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0..3000, 0.0f32..120.0f32)?;
    chart3.configure_mesh().draw()?;

    let colors = [RED, BLUE, GREEN, MAGENTA];

    for (i, (name, data)) in results.iter().enumerate() {
        let color = colors[i % colors.len()];
        
        chart1.draw_series(LineSeries::new(
            data.episodes.iter().zip(data.rewards.iter()).map(|(&x, &y)| (x, y)),
            &color,
        ))?
        .label(*name)
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));

        chart2.draw_series(LineSeries::new(
            data.episodes.iter().zip(data.accuracies.iter()).map(|(&x, &y)| (x, y)),
            &color,
        ))?
        .label(*name)
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));

        chart3.draw_series(LineSeries::new(
            data.episodes.iter().zip(data.iprs.iter()).map(|(&x, &y)| (x, y)),
            &color,
        ))?
        .label(format!("{} (IPR)", name))
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));
    }

    chart1.configure_series_labels().background_style(&WHITE.mix(0.8)).border_style(&BLACK).draw()?;
    chart2.configure_series_labels().background_style(&WHITE.mix(0.8)).border_style(&BLACK).draw()?;
    chart3.configure_series_labels().background_style(&WHITE.mix(0.8)).border_style(&BLACK).draw()?;

    root.present()?;
    println!("Graph saved to convergence_graph_rust.png");
    Ok(())
}
