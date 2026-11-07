//! Minesweeper board logic — pure data, no I/O, fully unit-testable.

use std::collections::VecDeque;

use rand::RngExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigResult {
    /// Dug into a mine — game lost.
    Boomed,
    /// Revealed this cell (and neighbours, if empty).
    Revealed,
    /// Cell was already revealed.
    AlreadyOpen,
    /// Cell is flagged; dig refused.
    Flagged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordOutcome {
    /// Cell is not a revealed number — nothing to do.
    NotApplicable,
    /// Flags around the cell don't match its number — refuses to dig.
    Mismatch,
    /// All unflagged hidden neighbours were revealed safely.
    Revealed,
    /// An unflagged neighbour was a mine — game lost.
    Boomed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    InProgress,
    Won,
    Lost,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cell {
    pub mine: bool,
    pub revealed: bool,
    pub flagged: bool,
    pub adjacent: u8,
}

pub struct Board {
    pub rows: usize,
    pub cols: usize,
    pub mines: usize,
    cells: Vec<Cell>,
    mines_placed: bool,
    revealed_count: usize,
}

impl Board {
    /// Creates an empty board. Mines are placed lazily on the first dig so
    /// that the first click is always safe.
    pub fn new(rows: usize, cols: usize, mines: usize) -> Self {
        assert!(mines < rows * cols, "mine count must leave at least 1 free cell");
        Self {
            rows,
            cols,
            mines,
            cells: vec![Cell::default(); rows * cols],
            mines_placed: false,
            revealed_count: 0,
        }
    }

    pub fn cell(&self, r: usize, c: usize) -> Cell {
        self.cells[r * self.cols + c]
    }

    pub fn status(&self) -> Status {
        if self.cells.iter().any(|c| c.mine && c.revealed) {
            Status::Lost
        } else if self.revealed_count == self.rows * self.cols - self.mines {
            Status::Won
        } else {
            Status::InProgress
        }
    }

    pub fn flags_placed(&self) -> usize {
        self.cells.iter().filter(|c| c.flagged).count()
    }

    /// Reveals every mine (used when the game is lost).
    pub fn reveal_all_mines(&mut self) {
        for cell in &mut self.cells {
            if cell.mine {
                cell.revealed = true;
            }
        }
    }

    /// On win: flags every remaining mine (classic cosmetic behaviour).
    pub fn auto_flag_remaining(&mut self) {
        for cell in &mut self.cells {
            if cell.mine && !cell.flagged {
                cell.flagged = true;
            }
        }
    }

    /// Digs the cell at `(r, c)`. Flood-fills empty regions automatically.
    pub fn dig(&mut self, r: usize, c: usize) -> DigResult {
        if r >= self.rows || c >= self.cols {
            return DigResult::AlreadyOpen;
        }
        if self.cell(r, c).flagged {
            return DigResult::Flagged;
        }
        if self.cell(r, c).revealed {
            return DigResult::AlreadyOpen;
        }
        if !self.mines_placed {
            self.place_mines(r, c);
        }
        if self.cell(r, c).mine {
            self.reveal_cell(r, c);
            return DigResult::Boomed;
        }
        self.flood_reveal(r, c);
        DigResult::Revealed
    }

    /// Toggles the flag on an unrevealed cell. Returns `false` if the cell is
    /// already revealed (cannot be flagged).
    pub fn toggle_flag(&mut self, r: usize, c: usize) -> bool {
        if r >= self.rows || c >= self.cols {
            return false;
        }
        let cell = &mut self.cells[r * self.cols + c];
        if cell.revealed {
            return false;
        }
        cell.flagged = !cell.flagged;
        true
    }

    /// Chording: dig every unflagged hidden neighbour of a revealed number
    /// cell, but only when the number of surrounding flags matches the
    /// number exactly. Flagged cells are never touched. If a dug neighbour
    /// turns out to be a mine, the game is lost (classic rules).
    pub fn chord(&mut self, r: usize, c: usize) -> ChordOutcome {
        if r >= self.rows || c >= self.cols {
            return ChordOutcome::NotApplicable;
        }
        let cell = self.cell(r, c);
        if !cell.revealed || cell.adjacent == 0 {
            return ChordOutcome::NotApplicable;
        }
        let flagged = self
            .neighbours(r, c)
            .filter(|&(nr, nc)| self.cell(nr, nc).flagged)
            .count();
        if flagged != cell.adjacent as usize {
            return ChordOutcome::Mismatch;
        }
        let targets: Vec<(usize, usize)> = self
            .neighbours(r, c)
            .filter(|&(nr, nc)| {
                let n = self.cell(nr, nc);
                !n.revealed && !n.flagged
            })
            .collect();
        for (nr, nc) in targets {
            if self.cell(nr, nc).mine {
                self.reveal_cell(nr, nc);
                return ChordOutcome::Boomed;
            }
            self.flood_reveal(nr, nc);
        }
        ChordOutcome::Revealed
    }

    /// Places all mines on random cells, excluding `(r, c)` so the first dig
    /// is always safe.
    fn place_mines(&mut self, safe_r: usize, safe_c: usize) {
        let mut rng = rand::rng();
        let mut candidates: Vec<usize> = (0..self.cells.len())
            .filter(|&i| i != safe_r * self.cols + safe_c)
            .collect();
        // Partial Fisher–Yates: pick `mines` distinct positions.
        let n = candidates.len();
        for i in 0..self.mines {
            let j = rng.random_range(i..n);
            candidates.swap(i, j);
            let (r, c) = (candidates[i] / self.cols, candidates[i] % self.cols);
            self.cells[candidates[i]].mine = true;
            let around: Vec<(usize, usize)> = self.neighbours(r, c).collect();
            for (nr, nc) in around {
                self.cells[nr * self.cols + nc].adjacent += 1;
            }
        }
        self.mines_placed = true;
    }

    fn reveal_cell(&mut self, r: usize, c: usize) {
        let cell = &mut self.cells[r * self.cols + c];
        if !cell.revealed {
            cell.revealed = true;
            self.revealed_count += 1;
        }
    }

    /// Iterative flood fill: reveal the cell, then keep expanding through
    /// zero-adjacency cells and their borders.
    fn flood_reveal(&mut self, start_r: usize, start_c: usize) {
        let mut queue = VecDeque::new();
        queue.push_back((start_r, start_c));
        while let Some((r, c)) = queue.pop_front() {
            if self.cell(r, c).revealed {
                continue;
            }
            self.reveal_cell(r, c);
            if self.cell(r, c).adjacent == 0 {
                for (nr, nc) in self.neighbours(r, c) {
                    let n = self.cell(nr, nc);
                    if !n.revealed && !n.flagged && !n.mine {
                        queue.push_back((nr, nc));
                    }
                }
            }
        }
    }

    /// All in-bounds neighbours of `(r, c)`, excluding the cell itself.
    fn neighbours(&self, r: usize, c: usize) -> impl Iterator<Item = (usize, usize)> {
        let (rows, cols) = (self.rows, self.cols);
        let (r0, c0) = (r.saturating_sub(1), c.saturating_sub(1));
        (r0..=(r + 1).min(rows - 1))
            .flat_map(move |nr| (c0..=(c + 1).min(cols - 1)).map(move |nc| (nr, nc)))
            .filter(move |&(nr, nc)| !(nr == r && nc == c))
    }

    /// Test-only: plant mines at fixed positions so tests are deterministic
    /// (the real game places mines randomly on first dig).
    #[cfg(test)]
    pub fn plant_mines(&mut self, positions: &[(usize, usize)]) {
        for &(r, c) in positions {
            self.cells[r * self.cols + c].mine = true;
            let around: Vec<(usize, usize)> = self.neighbours(r, c).collect();
            for (nr, nc) in around {
                self.cells[nr * self.cols + nc].adjacent += 1;
            }
        }
        self.mines_placed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_dig_is_always_safe_and_places_all_mines() {
        let mut b = Board::new(9, 9, 10);
        b.dig(4, 4);
        assert!(!b.cell(4, 4).mine);
        let mine_count = (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter(|&(r, c)| b.cell(r, c).mine)
            .count();
        assert_eq!(mine_count, 10);
        assert!(b.mines_placed);
    }

    #[test]
    fn dig_on_flagged_cell_is_refused() {
        let mut b = Board::new(5, 5, 3);
        assert!(b.toggle_flag(2, 2));
        assert_eq!(b.dig(2, 2), DigResult::Flagged);
        assert!(!b.cell(2, 2).revealed);
    }

    #[test]
    fn dig_on_revealed_cell_is_noop() {
        let mut b = Board::new(5, 5, 3);
        b.dig(1, 1);
        assert!(b.cell(1, 1).revealed);
        assert_eq!(b.dig(1, 1), DigResult::AlreadyOpen);
    }

    #[test]
    fn cannot_flag_a_revealed_cell() {
        let mut b = Board::new(5, 5, 3);
        b.dig(0, 0);
        assert!(!b.toggle_flag(0, 0));
        assert!(!b.cell(0, 0).flagged);
    }

    #[test]
    fn flood_fill_reveals_connected_region() {
        let mut b = Board::new(6, 6, 2);
        b.dig(0, 0);
        // Every revealed non-mine cell with adjacent==0 must have all its
        // neighbours revealed too (flood-fill invariant).
        for r in 0..6 {
            for c in 0..6 {
                let cell = b.cell(r, c);
                if cell.revealed && cell.adjacent == 0 {
                    for (nr, nc) in b.neighbours(r, c) {
                        assert!(
                            b.cell(nr, nc).revealed || b.cell(nr, nc).mine,
                            "unrevealed neighbour at {nr},{nc}"
                        );
                    }
                }
            }
        }
        assert!(b.cell(0, 0).revealed);
    }

    #[test]
    fn digging_a_mine_loses_the_game() {
        let mut b = Board::new(4, 4, 1);
        b.dig(0, 0);
        // Find the mine and dig it directly.
        let mine_at = (0..4)
            .flat_map(|r| (0..4).map(move |c| (r, c)))
            .find(|&(r, c)| b.cell(r, c).mine)
            .unwrap();
        b.reveal_all_mines();
        b.dig(mine_at.0, mine_at.1);
        assert_eq!(b.status(), Status::Lost);
    }

    #[test]
    fn revealing_every_safe_cell_wins_the_game() {
        let mut b = Board::new(2, 3, 1);
        b.dig(0, 0);
        for r in 0..2 {
            for c in 0..3 {
                if !b.cell(r, c).mine && !b.cell(r, c).revealed {
                    b.dig(r, c);
                }
            }
        }
        assert_eq!(b.status(), Status::Won);
    }

    #[test]
    fn adjacent_counts_match_neighbouring_mines() {
        let mut b = Board::new(9, 9, 10);
        b.dig(8, 8);
        for r in 0..9 {
            for c in 0..9 {
                let expected = b
                    .neighbours(r, c)
                    .filter(|&(nr, nc)| b.cell(nr, nc).mine)
                    .count() as u8;
                assert_eq!(b.cell(r, c).adjacent, expected, "cell {r},{c}");
            }
        }
    }

    #[test]
    fn flags_counter_works() {
        let mut b = Board::new(5, 5, 3);
        assert_eq!(b.flags_placed(), 0);
        b.toggle_flag(0, 0);
        b.toggle_flag(1, 1);
        assert_eq!(b.flags_placed(), 2);
        b.toggle_flag(0, 0);
        assert_eq!(b.flags_placed(), 1);
    }

    #[test]
    fn chord_on_unrevealed_or_zero_cell_does_nothing() {
        let mut b = Board::new(3, 3, 1);
        b.plant_mines(&[(2, 2)]);
        // Unrevealed cell.
        assert_eq!(b.chord(0, 0), ChordOutcome::NotApplicable);
        // Revealed zero cell (far from the mine): flood-open it first.
        b.dig(2, 0);
        assert!(b.cell(2, 0).revealed);
        assert_eq!(b.cell(2, 0).adjacent, 0);
        assert_eq!(b.chord(2, 0), ChordOutcome::NotApplicable);
    }

    #[test]
    fn chord_refuses_when_flags_do_not_match_number() {
        let mut b = Board::new(3, 4, 2);
        b.plant_mines(&[(0, 0), (0, 2)]);
        b.dig(1, 1); // (1,1) has 2 adjacent mines, stays a number cell
        assert_eq!(b.cell(1, 1).adjacent, 2);
        b.toggle_flag(1, 0); // only 1 of 2 flags so far
        assert_eq!(b.chord(1, 1), ChordOutcome::Mismatch);
        assert!(!b.cell(1, 2).revealed, "no neighbour may open on mismatch");
    }

    #[test]
    fn chord_opens_unflagged_neighbours_and_skips_flagged() {
        let mut b = Board::new(3, 4, 2);
        b.plant_mines(&[(0, 0), (0, 2)]);
        b.dig(1, 1);
        // Flag both mines — chord must open everything else around (1,1).
        b.toggle_flag(0, 0);
        b.toggle_flag(0, 2);
        assert_eq!(b.chord(1, 1), ChordOutcome::Revealed);
        for (nr, nc) in [(0, 1), (1, 0), (1, 2), (2, 0), (2, 1), (2, 2)] {
            assert!(b.cell(nr, nc).revealed, "neighbour {nr},{nc} should open");
        }
        // Flagged mines stay hidden and still flagged.
        assert!(!b.cell(0, 0).revealed && b.cell(0, 0).flagged);
        assert!(!b.cell(0, 2).revealed && b.cell(0, 2).flagged);
    }

    #[test]
    fn chord_with_wrong_flags_booms() {
        let mut b = Board::new(3, 4, 2);
        b.plant_mines(&[(0, 0), (0, 2)]);
        b.dig(1, 1);
        // One flag on a mine, one on a safe cell — flags "match" the number
        // but a mine is left unflagged next to (1,1).
        b.toggle_flag(0, 2);
        b.toggle_flag(2, 1);
        assert_eq!(b.chord(1, 1), ChordOutcome::Boomed);
        assert!(b.cell(0, 0).revealed, "the mine that was hit must show");
        assert_eq!(b.status(), Status::Lost);
    }

    #[test]
    fn chord_can_win_the_game() {
        let mut b = Board::new(2, 3, 1);
        b.plant_mines(&[(0, 0)]);
        b.dig(1, 1); // number cell adjacent to the single mine
        b.toggle_flag(0, 0);
        b.chord(1, 1); // opens every remaining safe cell
        assert_eq!(b.status(), Status::Won);
    }
}
