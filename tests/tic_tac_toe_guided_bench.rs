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
            [0, 1, 2], [3, 4, 5], [6, 7, 8],
            [0, 3, 6], [1, 4, 7], [2, 5, 8],
            [0, 4, 8], [2, 4, 6],
        ];
        for l in lines {
            if self.cells[l[0]] != Cell::Empty && self.cells[l[0]] == self.cells[l[1]] && self.cells[l[0]] == self.cells[l[2]] {
                return Some(self.cells[l[0]]);
            }
        }
        None
    }

    /// エキスパートのロジック: 反則を避け、勝てるなら勝ち、リーチがあれば防ぐ
    fn get_expert_move(&self, player: Cell) -> usize {
        let opponent = if player == Cell::X { Cell::O } else { Cell::X };

        // 1. 勝てる手があるか？
        for i in 0..9 {
            if self.cells[i] == Cell::Empty {
                let mut next_board = Board { cells: self.cells };
                next_board.cells[i] = player;
                if next_board.check_winner() == Some(player) { return i; }
            }
        }

        // 2. 相手のリーチを防ぐ手があるか？
        for i in 0..9 {
            if self.cells[i] == Cell::Empty {
                let mut next_board = Board { cells: self.cells };
                next_board.cells[i] = opponent;
                if next_board.check_winner() == Some(opponent) { return i; }
            }
        }

        // 3. 適当な空きマス（中心優先）
        if self.cells[4] == Cell::Empty { return 4; }
        let corners = [0, 2, 6, 8];
        for &c in &corners {
            if self.cells[c] == Cell::Empty { return c; }
        }
        for i in 0..9 {
            if self.cells[i] == Cell::Empty { return i; }
        }
        0
    }
}

#[test]
fn benchmark_tic_tac_toe_guided_evolution() {
    let mut ai_x = Singularity::new(19683, vec![9]);
    let mut ai_o = Singularity::new(19683, vec![9]);

    println!("
--- DS-Bench: Guided Tic-Tac-Toe Evolution ---");
    
    // ---------------------------------------------------------
    // Phase 1: Knowledge Injection (Expert Observation)
    // ---------------------------------------------------------
    println!("Phase 1: Injecting Initial Knowledge via Expert Observation...");
    for _ in 0..200 {
        let mut board = Board::new();
        // ランダムに数手進めた状態を作る
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let turns = (seed % 6) as usize;
        let mut cur_player = Cell::X;
        for t in 0..turns {
            let mv = board.get_expert_move(cur_player);
            board.cells[mv] = cur_player;
            cur_player = if cur_player == Cell::X { Cell::O } else { Cell::X };
        }

        if board.check_winner().is_none() && !board.is_full() {
            let state_idx = board.get_state_index(cur_player);
            let expert_move = board.get_expert_move(cur_player);
            
            // エキスパートの動きを観測して動機を逆算
            if cur_player == Cell::X {
                ai_x.observe_expert(state_idx, &[expert_move], 1.0);
            } else {
                ai_o.observe_expert(state_idx, &[expert_move], 1.0);
            }
        }
    }
    println!("Knowledge injection complete. (X-Rhyd: {:.2}, O-Rhyd: {:.2})", 
             ai_x.get_resonance_density(), ai_o.get_resonance_density());

    // ---------------------------------------------------------
    // Phase 2: Co-Evolution (Self-Play)
    // ---------------------------------------------------------
    println!("
Phase 2: Starting Co-Evolution with Guided Motives...");
    let mut x_wins = 0;
    let mut o_wins = 0;
    let mut invalid_moves = 0;
    let total_matches = 300;
    let report_interval = 50;

    for m in 1..=total_matches {
        let mut board = Board::new();
        let mut turn = Cell::X;
        let mut game_over = false;

        while !game_over {
            let current_ai = if turn == Cell::X { &mut ai_x } else { &mut ai_o };
            let state_idx = board.get_state_index(turn);
            
            let actions = current_ai.select_actions(state_idx);
            let move_idx = actions[0] as usize;

            if board.cells[move_idx] != Cell::Empty {
                current_ai.learn(-5.0);
                if turn == Cell::X { o_wins += 1; } else { x_wins += 1; }
                invalid_moves += 1;
                game_over = true;
            } else {
                board.cells[move_idx] = turn;
                if let Some(winner) = board.check_winner() {
                    if winner == Cell::X {
                        ai_x.learn(2.0); ai_o.learn(-2.0);
                        x_wins += 1;
                    } else {
                        ai_x.learn(-2.0); ai_o.learn(2.0);
                        o_wins += 1;
                    }
                    game_over = true;
                } else if board.is_full() {
                    ai_x.learn(0.5); ai_o.learn(0.5);
                    game_over = true;
                } else {
                    turn = if turn == Cell::X { Cell::O } else { Cell::X };
                }
            }
        }

        if m % report_interval == 0 {
            let win_rate_x = (x_wins as f64 / report_interval as f64) * 100.0;
            let win_rate_o = (o_wins as f64 / report_interval as f64) * 100.0;
            let invalid_rate = (invalid_moves as f64 / report_interval as f64) * 100.0;
            
            println!("Match {:03}-{:03} | X-Win: {:>5.1}% | O-Win: {:>5.1}% | Invalid: {:>5.1}% | Temp: {:.2}", 
                     m - report_interval + 1, m, win_rate_x, win_rate_o, invalid_rate, ai_x.system_temperature);
            x_wins = 0; o_wins = 0; invalid_moves = 0;
        }
    }

    println!("
Guided Evolution Summary:");
    println!("AI-X Final Resonance Density: {:.4} Rhyd", ai_x.get_resonance_density());
    println!("AI-O Final Resonance Density: {:.4} Rhyd", ai_o.get_resonance_density());
    
    // 初期知識があるため、最初から Invalid move が少ないはず
    assert!(ai_x.get_resonance_density() > 1.0, "AI should maintain structure from injection");
}
