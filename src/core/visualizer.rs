use plotters::prelude::*;
use super::mwso::MWSO;

pub struct Visualizer;

impl Visualizer {
    /// MWSOの波動状態を3D空間にプロットし、画像として保存する
    pub fn render_wave_snapshot(mwso: &MWSO, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
        
        // Dark Singularity スタイルの黒背景
        root.fill(&BLACK)?;

        let mut chart = ChartBuilder::on(&root)
            .margin(20)
            .caption("MWSO Wave-State Manifestation", ("sans-serif", 40).into_font().color(&WHITE))
            .build_cartesian_3d(0.0..512.0, -1.2..1.2, -1.2..1.2)?;

        // 視点の設定（斜め上から）
        chart.with_projection(|mut pb| {
            pb.yaw = 0.5;
            pb.pitch = 0.3;
            pb.scale = 0.7;
            pb.into_matrix()
        });

        // グリッド線の描画（控えめに）
        chart.configure_axes()
            .light_grid_style(&RGBColor(30, 30, 30))
            .draw()?;

        // 波動データのプロット
        // X: Index, Y: Real, Z: Imaginary
        let data: Vec<(f32, f32, f32)> = (0..mwso.dim)
            .map(|i| (i as f32, mwso.psi_real[i], mwso.psi_imag[i]))
            .collect();

        // 波動をネオンブルーの線で描画
        chart.draw_series(LineSeries::new(
            data.iter().map(|&(x, y, z)| (x as f64, y as f64, z as f64)),
            &CYAN,
        ))?;

        // 各成分を小さな点で強調
        chart.draw_series(data.iter().map(|&(x, y, z)| {
            Circle::new((x as f64, y as f64, z as f64), 2, Into::<ShapeStyle>::into(&CYAN).filled())
        }))?;

        root.present()?;
        Ok(())
    }
}
