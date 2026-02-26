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
        let lines = [[0,1,2],[3,4,5],[6,7,8],[0,3,6],[1,4,7],[2,5,8],[0,4,8],[2,4,6]];
        for l in lines {
            if self.cells[l[0]] != Cell::Empty && self.cells[l[0]] == self.cells[l[1]] && self.cells[l[0]] == self.cells[l[2]] {
                return Some(self.cells[l[0]]);
            }
        }
        None
    }

    fn get_expert_move(&self, player: Cell) -> usize {
        let opponent = if player == Cell::X { Cell::O } else { Cell::X };
        for i in 0..9 {
            if self.cells[i] == Cell::Empty {
                let mut nb = Board { cells: self.cells }; nb.cells[i] = player;
                if nb.check_winner() == Some(player) { return i; }
            }
        }
        for i in 0..9 {
            if self.cells[i] == Cell::Empty {
                let mut nb = Board { cells: self.cells }; nb.cells[i] = opponent;
                if nb.check_winner() == Some(opponent) { return i; }
            }
        }
        if self.cells[4] == Cell::Empty { return 4; }
        for &c in &[0, 2, 6, 8] { if self.cells[c] == Cell::Empty { return c; } }
        for i in 0..9 { if self.cells[i] == Cell::Empty { return i; } }
        0
    }
}

#[test]
fn benchmark_tic_tac_toe_o_specialist_evolution() {
    let mut ai_x = Singularity::new(19683, vec![9]);
    let mut ai_o = Singularity::new(19683, vec![9]);

    println!("
--- DS-Bench: Asymmetric Co-Evolution (O-Specialist) ---");
    
    // Phase 1: Asymmetric Education
    println!("Phase 1: Training AI-O as a Defensive Specialist...");
    
    // AI-X は通常の先攻学習 (500回)
    for _ in 0..500 {
        let mut board = Board::new();
        let state_idx = board.get_state_index(Cell::X);
        let mv = board.get_expert_move(Cell::X);
        ai_x.observe_expert(state_idx, &[mv], 0.8);
    }

    // AI-O は「Xが打った後」の状態を重点的に学習 (1500回)
    for i in 0..1500 {
        let mut board = Board::new();
        // Xにランダムあるいは戦略的な初手を打たせる
        let x_mv = if i % 3 == 0 { 4 } else { (i * 7) % 9 };
        if board.cells[x_mv] == Cell::Empty {
            board.cells[x_mv] = Cell::X;
            let state_idx = board.get_state_index(Cell::O);
            let o_mv = board.get_expert_move(Cell::O);
            // 強い強度で後攻の動きを刻み込む
            ai_o.observe_expert(state_idx, &[o_mv], 1.5);
        }
    }

    println!("Asymmetric Education Complete.");
    println!("Initial Rhyd | X: {:.2} | O: {:.2}", ai_x.get_resonance_density(), ai_o.get_resonance_density());

    // Phase 2: Co-Evolution
    println!("
Phase 2: Starting Co-Evolution...");
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
                    if winner == Cell::X { ai_x.learn(2.0); ai_o.learn(-2.0); x_wins += 1; }
                    else { ai_x.learn(-2.0); ai_o.learn(2.0); o_wins += 1; }
                    game_over = true;
                } else if board.is_full() {
                    ai_x.learn(0.5); ai_o.learn(0.5); draws += 1; game_over = true;
                } else {
                    turn = if turn == Cell::X { Cell::O } else { Cell::X };
                }
            }
        }

        if m % report_interval == 0 {
            let win_rate_x = (x_wins as f32 / report_interval as f32) * 100.0;
            let win_rate_o = (o_wins as f32 / report_interval as f32) * 100.0;
            let inv_rate = (invalid_moves as f32 / report_interval as f32) * 100.0;
            println!("Match {:03}-{:03} | X-Win: {:>5.1}% | O-Win: {:>5.1}% | Invalid: {:>5.1}% | X-Rhyd: {:.1} | O-Rhyd: {:.1}", 
                     m-report_interval+1, m, win_rate_x, win_rate_o, inv_rate, ai_x.get_resonance_density(), ai_o.get_resonance_density());
            x_wins = 0; o_wins = 0; draws = 0; invalid_moves = 0;
        }
    }
}
