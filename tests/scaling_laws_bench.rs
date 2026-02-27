use dark_singularity::core::mwso::MWSO;
use dark_singularity::core::singularity::Singularity;

/// 最適化された SNR 計算
fn calculate_interference_snr_optimized(mwso: &MWSO, patterns: &Vec<(Vec<f32>, Vec<f32>)>, target_idx: usize, total_energy_sq: f32) -> f32 {
    let dim = mwso.dim;
    let (target_re, target_im) = &patterns[target_idx];
    
    let mut s_re = 0.0_f64;
    let mut s_im = 0.0_f64;
    for j in 0..dim {
        s_re += target_re[j] as f64 * mwso.memory_psi_real[j] + target_im[j] as f64 * mwso.memory_psi_imag[j];
        s_im += target_re[j] as f64 * mwso.memory_psi_imag[j] - target_im[j] as f64 * mwso.memory_psi_real[j];
    }
    let signal_sq = (s_re.powi(2) + s_im.powi(2)) as f32;
    
    // Noise: 全エネルギーからターゲット信号成分を除いたもの
    let noise_floor_sq = (total_energy_sq - signal_sq).max(0.0) / (patterns.len() as f32);
    
    if noise_floor_sq < 1e-10 { return 100.0; }
    (signal_sq / noise_floor_sq).sqrt()
}

fn generate_random_phase_pattern(dim: usize, seed: usize) -> (Vec<f32>, Vec<f32>) {
    let mut re = vec![0.0; dim];
    let mut im = vec![0.0; dim];
    let inv_sqrt_dim = 1.0 / (dim as f32).sqrt();
    for i in 0..dim {
        let phase = (((i + seed * 123) as f32 * 0.618).rem_euclid(1.0)) * 2.0 * std::f32::consts::PI;
        re[i] = phase.cos() * inv_sqrt_dim;
        im[i] = phase.sin() * inv_sqrt_dim;
    }
    (re, im)
}

#[test]
fn benchmark_memory_capacity_scaling() {
    println!("\n=== Benchmark 1: Dimension (D) vs Superposition Capacity (N) ===");
    println!("Criterion: Interference SNR >= 5.0 (Crosstalk Limit)");

    let dimensions = vec![1024, 2048, 3072,4096, 8192, 16384];
    println!("{:<10} | {:<10} | {:<10} | {:<10}", "Dim (D)", "Max N", "N/D Ratio", "Scaling Log");
    println!("{}", "-".repeat(60));

    let mut prev_n = 1.0;
    let mut prev_d = 1.0;

    for &dim in &dimensions {
        let mut mwso = MWSO::new(dim);
        let mut patterns = Vec::new();
        let mut n = 0;

        loop {
            let next_n = n + 1;
            let (re, im) = generate_random_phase_pattern(dim, next_n);
            mwso.imprint_memory(&re, &im, 1.0);
            patterns.push((re, im));

            // 最適化: 全エネルギーを1回だけ計算
            let mut total_energy_sq = 0.0_f64;
            for j in 0..dim {
                total_energy_sq += mwso.memory_psi_real[j].powi(2) + mwso.memory_psi_imag[j].powi(2);
            }
            let total_energy_sq = total_energy_sq as f32;

            // チェックの高速化: 直近のパターンと過去の代表1点をサンプリング
            let snr_latest = calculate_interference_snr_optimized(&mwso, &patterns, next_n - 1, total_energy_sq);
            let snr_old = if next_n > 1 { calculate_interference_snr_optimized(&mwso, &patterns, 0, total_energy_sq) } else { 100.0 };

            if snr_latest < 5.0 || snr_old < 5.0 {
                break;
            }
            n = next_n;
            // 次元に応じた現実的な上限を設定（停滞防止）
            if n >= dim * 64 { break; } // かなり大きくする
            
            // 進捗が分かりにくいので、たまに出力
            if n % 500 == 0 {
                print!(".");
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
        }

        let ratio = n as f32 / dim as f32;
        let scaling_exponent = if prev_n > 0.0 { (n as f32 / prev_n).log2() / (dim as f32 / prev_d).log2() } else { 0.0 };
        
        println!("\n{:<10} | {:<10} | {:<10.4} | O(D^{:.2})", dim, n, ratio, scaling_exponent);
        
        prev_n = n as f32;
        prev_d = dim as f32;
    }
}

#[test]
fn benchmark_rhyd_crystallization() {
    println!("\n=== Benchmark 2: Rhyd vs Prediction Stability (with Annealing) ===");
    let mut ai = Singularity::new(100, vec![10]);
    let action_size = 10;
    println!("{:<10} | {:<10} | {:<10} | {:<15} | {:<10}", "Epoch", "Temp", "Rhyd", "Confidence", "Stable?");
    println!("{}", "-".repeat(65));

    for epoch in 1..=200 {
        let state_idx = epoch % 5;
        let target_action = (state_idx * 3) % action_size;
        ai.system_temperature = (2.0 * (1.0 - epoch as f32 / 180.0)).max(0.01);
        let _ = ai.select_actions(state_idx);
        ai.learn(if ai.last_actions[0] % action_size == target_action { 2.0 } else { -2.0 });

        if epoch % 20 == 0 {
            let rhyd = ai.get_resonance_density();
            let scores = ai.mwso.get_action_scores(0, action_size, 0.0, &vec![0.0; ai.mwso.dim]);
            let max_score = scores.iter().cloned().fold(0./0., f32::max);
            let sum_score: f32 = scores.iter().sum();
            let confidence = if sum_score > 0.0 { max_score / sum_score } else { 0.0 };
            println!("{:<10} | {:<10.2} | {:<10.2} | {:<15.4} | {:<10}", 
                     epoch, ai.system_temperature, rhyd, confidence, if confidence > 0.90 { "CRYSTAL" } else { "FLUID" });
        }
    }
}

#[test]
fn benchmark_thermal_phase_transition() {
    println!("\n=== Benchmark 3: Temperature (T) vs Convergence Speed (Hard Task) ===");
    let temperatures = vec![2.0, 1.2, 0.8, 0.4, 0.1];
    println!("{:<10} | {:<15} | {:<10}", "Temp (T)", "Epochs to Conv.", "Success Rate");
    println!("{}", "-".repeat(45));

    for &temp in &temperatures {
        let mut ai = Singularity::new(100, vec![5]);
        let mut success_streak = 0;
        let mut converged_at = 0;
        for epoch in 1..=1000 {
            let state_idx = epoch % 20;
            let target_action = (state_idx * 3) % 5;
            ai.system_temperature = temp;
            let actions = ai.select_actions(state_idx);
            if actions[0] as usize == target_action {
                ai.learn(1.5);
                success_streak += 1;
            } else {
                ai.learn(-1.0);
                success_streak = 0;
            }
            if success_streak >= 40 {
                converged_at = epoch;
                break;
            }
        }
        let result = if converged_at > 0 { format!("{}", converged_at) } else { "FAILED".to_string() };
        println!("{:<10.1} | {:<15} | {:<10}", temp, result, if converged_at > 0 { "100%" } else { "0%" });
    }
}

#[test]
fn benchmark_thermal_scaling_laws() {
    println!("\n=== Benchmark 4: Thermal Phase Transition & Scaling Laws ===");
    println!("Goal: Identify Tc and scaling laws via convergence time (tau)");
    println!("Metric: tau = epochs to converge | Tc = highest T where tau <= min_tau * 1.1\n");

    let dims = vec![1024, 2048, 4096];
    let num_t_points = 30;
    let t_max: f32 = 2.0;
    let t_min: f32 = 0.01;
    let max_epochs = 3000;
    let success_streak_target = 30;

    let mut scaling_data: Vec<(usize, f32, Vec<(f32, Option<usize>)>)> = Vec::new();

    for &dim in &dims {
        println!("--- Dimension D = {} ---", dim);
        println!("{:<10} | {:<12} | {:<10}", "Temp (T)", "Conv Time", "Status");
        println!("{}", "-".repeat(38));

        let action_size = match dim {
            1024 => 16,
            2048 => 32,
            4096 => 64,
            _ => 16,
        };

        let mut dim_results: Vec<(f32, Option<usize>)> = Vec::new();

        for i in 0..num_t_points {
            let temp = t_max * (t_min / t_max).powf(i as f32 / (num_t_points - 1) as f32);

            let mut ai = Singularity::new(20, vec![action_size]);
            ai.temperature_locked = true;
            ai.system_temperature = temp;

            let mut converged_at = None;
            let mut success_streak = 0;

            for epoch in 1..=max_epochs {
                let state_idx = epoch % 10;
                let target_action = (state_idx * 7) % action_size;

                let actions = ai.select_actions(state_idx);
                if actions[0] as usize == target_action {
                    ai.learn(2.1);
                    success_streak += 1;
                } else {
                    ai.learn(-1.5);
                    success_streak = 0;
                }

                if success_streak >= success_streak_target {
                    converged_at = Some(epoch);
                    break;
                }
            }

            let conv_str = converged_at
                .map(|e| e.to_string())
                .unwrap_or_else(|| "∞".to_string());
            let status = if converged_at.is_some() { "OK" } else { "FAIL" };

            println!("{:<10.3} | {:<12} | {}", temp, conv_str, status);
            dim_results.push((temp, converged_at));
        }

        // 収束時間の最小値を求める
        let min_tau = dim_results.iter()
            .filter_map(|(_, conv)| *conv)
            .min()
            .unwrap_or(max_epochs);

        // Tc = min_tau * 1.1以内になる最高温度
        let tc_guess = dim_results.iter()
            .filter_map(|(t, conv)| {
                conv.filter(|&tau| tau <= (min_tau as f32 * 1.1) as usize)
                    .map(|_| *t)
            })
            .fold(0.0f32, f32::max);

        println!(">> min_tau = {}, Tc ~ {:.3}\n", min_tau, tc_guess);
        scaling_data.push((dim, tc_guess, dim_results));
    }

    // スケーリング解析
    println!("=== Scaling Analysis ===");
    println!("{:<10} | {:<10} | {:<10}", "Dim (D)", "Tc", "min_tau");
    println!("{}", "-".repeat(35));
    for (dim, tc, results) in &scaling_data {
        let min_tau = results.iter()
            .filter_map(|(_, conv)| *conv)
            .min()
            .unwrap_or(max_epochs);
        println!("{:<10} | {:<10.3} | {:<10}", dim, tc, min_tau);
    }

    if scaling_data.len() >= 2 {
        let (d1, tc1, _) = &scaling_data[0];
        let (d_last, tc_last, _) = &scaling_data[scaling_data.len() - 1];
        if *tc1 > 0.0 && *tc_last > 0.0 {
            let alpha = (tc_last / tc1).ln() / (*d_last as f32 / *d1 as f32).ln();
            println!("Tc ~ D^alpha: alpha = {:.4}", alpha);
        } else {
            println!("Tc not identified");
        }
    }

    // 臨界発散チェック: tau ~ |T - Tc|^-nu
    // T > Tc の点だけを使う（高温側のみ）
    println!("\n=== Critical Divergence: tau ~ |T - Tc|^-nu (T > Tc only) ===");
    for (dim, tc, results) in &scaling_data {
        if *tc <= 0.0 { continue; }
        println!("D = {} (Tc = {:.3}):", dim, tc);
        println!("  {:<12} | {:<10} | {:<10} | {:<12}", "|T - Tc|", "T", "tau", "ln(tau)");
        println!("  {}", "-".repeat(52));

        // T > Tc の点だけ抽出してソート
        let mut above_tc: Vec<(f32, f32, usize)> = results.iter()
            .filter_map(|(t, conv)| {
                if *t > *tc {
                    Some((*t - *tc, *t, conv.unwrap_or(max_epochs)))
                } else {
                    None
                }
            })
            .collect();
        above_tc.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        for (dist, t, tau) in &above_tc {
            println!("  {:<12.4} | {:<10.3} | {:<10} | {:<12.4}",
                dist, t, tau, (*tau as f32).ln());
        }

        // nu推定：log(tau) vs log(|T-Tc|) の傾きから
        // tau ~ |T-Tc|^-nu → log(tau) = -nu * log(|T-Tc|) + const
        // 最小二乗法で傾きを求める
        if above_tc.len() >= 3 {
            let n = above_tc.len() as f32;
            let log_dist: Vec<f32> = above_tc.iter()
                .map(|(dist, _, _)| dist.ln())
                .collect();
            let log_tau: Vec<f32> = above_tc.iter()
                .map(|(_, _, tau)| (*tau as f32).ln())
                .collect();

            let sum_x: f32 = log_dist.iter().sum();
            let sum_y: f32 = log_tau.iter().sum();
            let sum_xx: f32 = log_dist.iter().map(|x| x * x).sum();
            let sum_xy: f32 = log_dist.iter().zip(log_tau.iter()).map(|(x, y)| x * y).sum();

            let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
            let nu = -slope; // tau ~ |T-Tc|^-nu なので傾きの符号を反転

            println!("  Estimated nu (least squares, {} points): {:.4}", above_tc.len(), nu);
            if nu > 0.0 {
                println!("  → 臨界発散あり (nu > 0)");
            } else {
                println!("  → 臨界発散なし (nu <= 0) - クロスオーバー系の可能性");
            }
        } else {
            println!("  (T > Tc の点が少なすぎる - t_maxを上げるか温度範囲を広げる)");
        }
        println!();
    }
}