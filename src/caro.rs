//! Pure Caro (Gomoku) game logic and heuristic AI — no I/O, 100% testable.
//!
//! Played on a 15x15 grid. A player wins by forming an unbroken line of
//! 5 stones horizontally, vertically, or diagonally.

pub const BOARD_SIZE: usize = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    X,
    O,
}

impl Player {
    pub fn opponent(self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }

    #[allow(dead_code)]
    pub fn symbol(self) -> &'static str {
        match self {
            Player::X => "X",
            Player::O => "O",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Won(Player),
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceResult {
    Success,
    InvalidPosition,
    CellOccupied,
    GameOver,
}

#[derive(Debug, Clone)]
pub struct CaroBoard {
    grid: [[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    turn: Player,
    status: GameStatus,
    winning_line: Option<Vec<(usize, usize)>>,
    moves_count: usize,
    last_move: Option<(usize, usize)>,
}

impl Default for CaroBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl CaroBoard {
    pub fn new() -> Self {
        Self {
            grid: [[None; BOARD_SIZE]; BOARD_SIZE],
            turn: Player::X,
            status: GameStatus::InProgress,
            winning_line: None,
            moves_count: 0,
            last_move: None,
        }
    }

    pub fn cell(&self, row: usize, col: usize) -> Option<Player> {
        if row < BOARD_SIZE && col < BOARD_SIZE {
            self.grid[row][col]
        } else {
            None
        }
    }

    pub fn turn(&self) -> Player {
        self.turn
    }

    pub fn status(&self) -> GameStatus {
        self.status
    }

    pub fn moves_count(&self) -> usize {
        self.moves_count
    }

    pub fn last_move(&self) -> Option<(usize, usize)> {
        self.last_move
    }

    pub fn winning_line(&self) -> Option<&[(usize, usize)]> {
        self.winning_line.as_deref()
    }

    /// Attempts to place a stone for the current player at (row, col).
    pub fn place(&mut self, row: usize, col: usize) -> PlaceResult {
        if self.status != GameStatus::InProgress {
            return PlaceResult::GameOver;
        }
        if row >= BOARD_SIZE || col >= BOARD_SIZE {
            return PlaceResult::InvalidPosition;
        }
        if self.grid[row][col].is_some() {
            return PlaceResult::CellOccupied;
        }

        let p = self.turn;
        self.grid[row][col] = Some(p);
        self.moves_count += 1;
        self.last_move = Some((row, col));

        if let Some(line) = self.check_win_at(row, col, p) {
            self.status = GameStatus::Won(p);
            self.winning_line = Some(line);
        } else if self.moves_count == BOARD_SIZE * BOARD_SIZE {
            self.status = GameStatus::Draw;
        } else {
            self.turn = self.turn.opponent();
        }

        PlaceResult::Success
    }

    /// Checks if placing stone of player `p` at (row, col) creates a line of 5 or more.
    fn check_win_at(&self, row: usize, col: usize, p: Player) -> Option<Vec<(usize, usize)>> {
        const DIRECTIONS: [(isize, isize); 4] = [
            (0, 1),  // Horizontal
            (1, 0),  // Vertical
            (1, 1),  // Diagonal down-right \
            (1, -1), // Diagonal down-left /
        ];

        for (dr, dc) in DIRECTIONS {
            let mut line = vec![(row, col)];

            // Forward direction
            let mut r = row as isize + dr;
            let mut c = col as isize + dc;
            while r >= 0 && r < BOARD_SIZE as isize && c >= 0 && c < BOARD_SIZE as isize {
                if self.grid[r as usize][c as usize] == Some(p) {
                    line.push((r as usize, c as usize));
                    r += dr;
                    c += dc;
                } else {
                    break;
                }
            }

            // Backward direction
            let mut r = row as isize - dr;
            let mut c = col as isize - dc;
            while r >= 0 && r < BOARD_SIZE as isize && c >= 0 && c < BOARD_SIZE as isize {
                if self.grid[r as usize][c as usize] == Some(p) {
                    line.push((r as usize, c as usize));
                    r -= dr;
                    c -= dc;
                } else {
                    break;
                }
            }

            if line.len() >= 5 {
                return Some(line);
            }
        }
        None
    }

    /// Heuristic AI to compute the best move for the current player.
    pub fn find_best_move(&self) -> Option<(usize, usize)> {
        if self.status != GameStatus::InProgress {
            return None;
        }

        // If board is empty, start at the exact center (7, 7)
        if self.moves_count == 0 {
            return Some((BOARD_SIZE / 2, BOARD_SIZE / 2));
        }

        let me = self.turn;
        let opp = me.opponent();

        let mut best_score = i64::MIN;
        let mut best_move = None;

        for r in 0..BOARD_SIZE {
            for c in 0..BOARD_SIZE {
                if self.grid[r][c].is_some() {
                    continue;
                }

                // Quick pruning: only consider cells within distance 2 of existing stones
                if !self.has_nearby_stone(r, c, 2) {
                    continue;
                }

                let attack = evaluate_point(self, r, c, me);
                let defense = evaluate_point(self, r, c, opp);

                // Total score: offense is weighted slightly higher than defense,
                // plus a small tie-breaker preference for the center.
                let center_dist = (r as isize - 7).abs() + (c as isize - 7).abs();
                let center_bonus = (14 - center_dist) as i64;
                let score = attack * 12 / 10 + defense + center_bonus;

                if score > best_score {
                    best_score = score;
                    best_move = Some((r, c));
                }
            }
        }

        // Fallback: if no nearby cells scored (rare edge cases), find any empty cell
        best_move.or_else(|| {
            for r in 0..BOARD_SIZE {
                for c in 0..BOARD_SIZE {
                    if self.grid[r][c].is_none() {
                        return Some((r, c));
                    }
                }
            }
            None
        })
    }

    fn has_nearby_stone(&self, row: usize, col: usize, dist: usize) -> bool {
        let r_min = row.saturating_sub(dist);
        let r_max = (row + dist).min(BOARD_SIZE - 1);
        let c_min = col.saturating_sub(dist);
        let c_max = (col + dist).min(BOARD_SIZE - 1);

        for r in r_min..=r_max {
            for c in c_min..=c_max {
                if self.grid[r][c].is_some() {
                    return true;
                }
            }
        }
        false
    }
}

/// Evaluates the tactical value of placing player `p` at (row, col).
fn evaluate_point(board: &CaroBoard, row: usize, col: usize, p: Player) -> i64 {
    const DIRECTIONS: [(isize, isize); 4] = [(0, 1), (1, 0), (1, 1), (1, -1)];

    let mut total = 0i64;

    for (dr, dc) in DIRECTIONS {
        let mut count = 1;
        let mut open_ends = 0;

        // Positive direction
        let mut r = row as isize + dr;
        let mut c = col as isize + dc;
        while r >= 0 && r < BOARD_SIZE as isize && c >= 0 && c < BOARD_SIZE as isize {
            if board.grid[r as usize][c as usize] == Some(p) {
                count += 1;
                r += dr;
                c += dc;
            } else {
                if board.grid[r as usize][c as usize].is_none() {
                    open_ends += 1;
                }
                break;
            }
        }

        // Negative direction
        let mut r = row as isize - dr;
        let mut c = col as isize - dc;
        while r >= 0 && r < BOARD_SIZE as isize && c >= 0 && c < BOARD_SIZE as isize {
            if board.grid[r as usize][c as usize] == Some(p) {
                count += 1;
                r -= dr;
                c -= dc;
            } else {
                if board.grid[r as usize][c as usize].is_none() {
                    open_ends += 1;
                }
                break;
            }
        }

        // Score this line
        total += match count {
            n if n >= 5 => 100_000_000,
            4 if open_ends == 2 => 10_000_000,
            4 if open_ends == 1 => 100_000,
            3 if open_ends == 2 => 50_000,
            3 if open_ends == 1 => 1_000,
            2 if open_ends == 2 => 500,
            2 if open_ends == 1 => 50,
            _ => 1,
        };
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_board_starts_with_player_x() {
        let board = CaroBoard::new();
        assert_eq!(board.turn(), Player::X);
        assert_eq!(board.status(), GameStatus::InProgress);
        assert_eq!(board.moves_count(), 0);
        assert_eq!(board.last_move(), None);
    }

    #[test]
    fn alternating_turns_and_cell_occupation() {
        let mut board = CaroBoard::new();
        assert_eq!(board.place(7, 7), PlaceResult::Success);
        assert_eq!(board.cell(7, 7), Some(Player::X));
        assert_eq!(board.turn(), Player::O);
        assert_eq!(board.moves_count(), 1);

        // Cannot place on occupied cell
        assert_eq!(board.place(7, 7), PlaceResult::CellOccupied);
        assert_eq!(board.turn(), Player::O);

        // O places on adjacent cell
        assert_eq!(board.place(7, 8), PlaceResult::Success);
        assert_eq!(board.cell(7, 8), Some(Player::O));
        assert_eq!(board.turn(), Player::X);
    }

    #[test]
    fn out_of_bounds_move_is_rejected() {
        let mut board = CaroBoard::new();
        assert_eq!(board.place(15, 0), PlaceResult::InvalidPosition);
        assert_eq!(board.place(0, 15), PlaceResult::InvalidPosition);
    }

    #[test]
    fn horizontal_five_in_a_row_wins() {
        let mut board = CaroBoard::new();
        // X places at (7, 0..4), O places at (8, 0..3)
        for c in 0..4 {
            board.place(7, c); // X
            board.place(8, c); // O
        }
        assert_eq!(board.status(), GameStatus::InProgress);
        board.place(7, 4); // X 5th stone
        assert_eq!(board.status(), GameStatus::Won(Player::X));
        assert!(board.winning_line().is_some());
        assert_eq!(board.winning_line().unwrap().len(), 5);

        // Cannot move after game over
        assert_eq!(board.place(0, 0), PlaceResult::GameOver);
    }

    #[test]
    fn vertical_five_in_a_row_wins() {
        let mut board = CaroBoard::new();
        for r in 0..4 {
            board.place(r, 7); // X
            board.place(r, 8); // O
        }
        board.place(4, 7); // X 5th
        assert_eq!(board.status(), GameStatus::Won(Player::X));
    }

    #[test]
    fn diagonal_down_right_wins() {
        let mut board = CaroBoard::new();
        for i in 0..4 {
            board.place(i, i); // X
            board.place(i, i + 1); // O
        }
        board.place(4, 4); // X 5th
        assert_eq!(board.status(), GameStatus::Won(Player::X));
    }

    #[test]
    fn diagonal_down_left_wins() {
        let mut board = CaroBoard::new();
        for i in 0..4 {
            board.place(i, 10 - i); // X
            board.place(i, 9 - i); // O
        }
        board.place(4, 6); // X 5th
        assert_eq!(board.status(), GameStatus::Won(Player::X));
    }

    #[test]
    fn ai_starts_at_center_on_empty_board() {
        let board = CaroBoard::new();
        assert_eq!(board.find_best_move(), Some((7, 7)));
    }

    #[test]
    fn ai_blocks_immediate_opponent_win() {
        let mut board = CaroBoard::new();
        // X has 4 in a row horizontally at (7, 2), (7, 3), (7, 4), (7, 5)
        board.place(7, 2); // X
        board.place(0, 0); // O
        board.place(7, 3); // X
        board.place(0, 1); // O
        board.place(7, 4); // X
        board.place(0, 2); // O
        board.place(7, 5); // X

        // Turn is O (the bot). X threatens to win at (7, 1) or (7, 6).
        assert_eq!(board.turn(), Player::O);
        let best = board.find_best_move().unwrap();
        assert!(best == (7, 1) || best == (7, 6), "AI must block the open 4 at (7,1) or (7,6), but chose {:?}", best);
    }

    #[test]
    fn ai_takes_immediate_win_over_defense() {
        let mut board = CaroBoard::new();
        // O has 4 in a row at (5, 1..=4), X has 4 in a row at (7, 1..=4)
        board.place(7, 1); // X
        board.place(5, 1); // O
        board.place(7, 2); // X
        board.place(5, 2); // O
        board.place(7, 3); // X
        board.place(5, 3); // O
        board.place(7, 4); // X
        board.place(5, 4); // O

        // Waste a move for X
        board.place(0, 0); // X

        // Turn is O. O can win at (5, 0) or (5, 5).
        assert_eq!(board.turn(), Player::O);
        let best = board.find_best_move().unwrap();
        assert!(best == (5, 0) || best == (5, 5), "AI must take winning move at (5,0) or (5,5), chose {:?}", best);
    }
}
