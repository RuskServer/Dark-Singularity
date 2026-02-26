use dark_singularity::core::singularity::Singularity;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Cell { Empty, X, O }

struct Board {
    cells: [Cell; 9],
}

impl Board {
    fn new() -> Self {
        Self { cells: [Cell::Empty; 9] }
    }

    /// ボードの状態を Singularity 用のインデックス（3進法）に変換
    /// player から見た視点で正規化（1: 自分, 2: 相手）
    fn get_state_index(&self, player: Cell) -> usize {
        let mut idx = 0;
        let mut p = 1;
        for &c in &self.cells {
            let val = match c {
                Cell::Empty => 0,
                c if c == player => 1,
                _ => 2,
            };
            idx += val * p;
            p *= 3;
        }
        idx
    }

    fn is_full(&self) -> bool {
        self.cells.iter().all(|&c| c != Cell::Empty)
    }

    fn check_winner(&self) -> Option<Cell> {
        let lines = [
            [0, 1, 2], [3, 4, 5], [6, 7, 8], // rows
            [0, 3, 6], [1, 4, 7], [2, 5, 8], // cols
            [0, 4, 8], [2, 4, 6],            // diags
        ];
        for l in lines {
            if self.cells[l[0]] != Cell::Empty && self.cells[l[0]] == self.cells[l[1]] && self.cells[l[0]] == self.cells[l[2]] {
                return Some(self.cells[l[0]]);
            }
        }
        None
    }
}

#[test]
fn benchmark_tic_tac_toe_evolution() {
    // 状態数 3^9 = 19683, アクション数 9 (マス目)
    let mut ai_x = Singularity::new(19683, vec![9]);
    let mut ai_o = Singularity::new(19683, vec![9]);

    println!("
--- DS-Bench: Tic-Tac-Toe Co-Evolution ---");
    println!("Two AIs playing against each other for 500 matches.");

    let mut x_wins = 0;
    let mut o_wins = 0;
    let mut draws = 0;
    let mut invalid_moves = 0;

    let total_matches = 500;
    let report_interval = 50;

    for m in 1..=total_matches {
        let mut board = Board::new();
        let mut turn = Cell::X;
        let mut game_over = false;

        // 1試合のループ
        while !game_over {
            let current_ai = if turn == Cell::X { &mut ai_x } else { &mut ai_o };
            let state_idx = board.get_state_index(turn);
            
            let actions = current_ai.select_actions(state_idx);
            let move_idx = actions[0] as usize;

            // ルールチェック
            if board.cells[move_idx] != Cell::Empty {
                // 反則負け
                current_ai.learn(-5.0);
                if turn == Cell::X { o_wins += 1; } else { x_wins += 1; }
                invalid_moves += 1;
                game_over = true;
            } else {
                board.cells[move_idx] = turn;
                
                if let Some(winner) = board.check_winner() {
                    if winner == Cell::X {
                        ai_x.learn(2.0);
                        ai_o.learn(-2.0);
                        x_wins += 1;
                    } else {
                        ai_x.learn(-2.0);
                        ai_o.learn(2.0);
                        o_wins += 1;
                    }
                    game_over = true;
                } else if board.is_full() {
                    ai_x.learn(0.5);
                    ai_o.learn(0.5);
                    draws += 1;
                    game_over = true;
                } else {
                    // 次のターンへ
                    turn = if turn == Cell::X { Cell::O } else { Cell::X };
                }
            }
        }

        if m % report_interval == 0 {
            let win_rate_x = (x_wins as f32 / report_interval as f32) * 100.0;
            let win_rate_o = (o_wins as f32 / report_interval as f32) * 100.0;
            let invalid_rate = (invalid_moves as f32 / report_interval as f32) * 100.0;
            
            println!("Match {:03}-{:03} | X-Win: {:>5.1}% | O-Win: {:>5.1}% | Invalid: {:>5.1}% | X-Rhyd: {:.2}", 
                     m - report_interval + 1, m, win_rate_x, win_rate_o, invalid_rate, ai_x.get_resonance_density());
            
            x_wins = 0;
            o_wins = 0;
            draws = 0;
            invalid_moves = 0;
        }
    }

    println!("
Evolution Summary:");
    println!("AI-X Final Resonance Density: {:.4} Rhyd", ai_x.get_resonance_density());
    println!("AI-O Final Resonance Density: {:.4} Rhyd", ai_o.get_resonance_density());
    
    // 進化の証拠：後半になるにつれ Invalid Move（反則）が激減するはず
    // また、共鳴密度(Rhyd)が初期状態(0.0付近)から有意に上昇していることを確認
    assert!(ai_x.get_resonance_density() > 0.5, "AI should evolve significant internal structure");
}
