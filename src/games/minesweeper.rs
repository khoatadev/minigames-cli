//! Minesweeper — terminal UI and game loop.
//!
//! Controls: arrow keys move the cursor, `X` digs, `Z` toggles a flag,
//! `R` restarts, `Q`/`Esc` returns to the menu.

use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::cursor::MoveTo;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::queue;
use crossterm::style::{Color, Print, SetBackgroundColor, Stylize};

use crate::board::{Board, DigResult, Status};
use crate::term;

pub struct Difficulty {
    pub name: &'static str,
    pub rows: usize,
    pub cols: usize,
    pub mines: usize,
}

pub const DIFFICULTIES: [Difficulty; 3] = [
    Difficulty { name: "Dễ", rows: 9, cols: 9, mines: 10 },
    Difficulty { name: "Vừa", rows: 16, cols: 16, mines: 40 },
    Difficulty { name: "Khó", rows: 16, cols: 30, mines: 99 },
];

enum Action {
    Continue,
    Restart,
    Quit,
}

struct Game {
    board: Board,
    cursor: (usize, usize),
    started: Instant,
    ended: Option<Instant>,
}

impl Game {
    fn new(diff: &Difficulty) -> Self {
        Self {
            board: Board::new(diff.rows, diff.cols, diff.mines),
            cursor: (0, 0),
            started: Instant::now(),
            ended: None,
        }
    }

    fn elapsed(&self) -> Duration {
        self.ended.unwrap_or_else(Instant::now) - self.started
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.kind != KeyEventKind::Press {
            return Action::Continue;
        }
        if self.ended.is_some() {
            return match key.code {
                KeyCode::Char('r' | 'R') => Action::Restart,
                KeyCode::Char('q' | 'Q') | KeyCode::Esc => Action::Quit,
                _ => Action::Continue,
            };
        }
        let (r, c) = self.cursor;
        match key.code {
            KeyCode::Left if c > 0 => self.cursor.1 -= 1,
            KeyCode::Right if c + 1 < self.board.cols => self.cursor.1 += 1,
            KeyCode::Up if r > 0 => self.cursor.0 -= 1,
            KeyCode::Down if r + 1 < self.board.rows => self.cursor.0 += 1,
            KeyCode::Char('x' | 'X') => self.dig(),
            KeyCode::Char('z' | 'Z') => {
                self.board.toggle_flag(r, c);
            }
            KeyCode::Char('r' | 'R') => return Action::Restart,
            KeyCode::Char('q' | 'Q') | KeyCode::Esc => return Action::Quit,
            _ => {}
        }
        Action::Continue
    }

    fn dig(&mut self) {
        let (r, c) = self.cursor;
        match self.board.dig(r, c) {
            DigResult::Boomed => {
                self.board.reveal_all_mines();
                self.ended = Some(Instant::now());
            }
            DigResult::Revealed if self.board.status() == Status::Won => {
                self.board.auto_flag_remaining();
                self.ended = Some(Instant::now());
            }
            _ => {}
        }
    }

    fn render(&self, out: &mut impl Write, diff: &Difficulty) -> io::Result<()> {
        term::begin_frame(out)?;
        let flags = self.board.flags_placed();
        let mines_left = self.board.mines as isize - flags as isize;

        // Header
        queue!(out, Print(format!("DÒ MÌN — {}   ", diff.name)))?;
        queue!(out, Print("Mìn: ".with(Color::DarkGrey)), Print(format!("{mines_left} ")))?;
        queue!(out, Print("Thời gian: ".with(Color::DarkGrey)), Print(mmss(self.elapsed())))?;

        // Board
        for r in 0..self.board.rows {
            queue!(out, MoveTo(1, (r + 2) as u16))?;
            for c in 0..self.board.cols {
                let on_cursor = self.cursor == (r, c);
                if on_cursor {
                    queue!(out, SetBackgroundColor(Color::DarkGrey))?;
                }
                self.draw_cell(out, r, c)?;
                if on_cursor {
                    queue!(out, SetBackgroundColor(Color::Reset))?;
                }
            }
        }

        // Status / footer
        let status_row = (self.board.rows + 3) as u16;
        queue!(out, MoveTo(1, status_row))?;
        match self.board.status() {
            Status::Lost => {
                queue!(out, Print("BẠN THUA!  ".with(Color::Red)))?;
                queue!(out, Print("R chơi lại, Q về menu".with(Color::DarkGrey)))?;
            }
            Status::Won => {
                queue!(out, Print(format!("BẠN THẮNG! Thời gian: {}  ", mmss(self.elapsed())).with(Color::Green)))?;
                queue!(out, Print("R chơi lại, Q về menu".with(Color::DarkGrey)))?;
            }
            Status::InProgress => {
                queue!(out, Print("X đào   Z cắm/gỡ cờ   R chơi lại   Q về menu".with(Color::DarkGrey)))?;
            }
        }
        out.flush()
    }

    fn draw_cell(&self, out: &mut impl Write, r: usize, c: usize) -> io::Result<()> {
        let cell = self.board.cell(r, c);
        if cell.flagged && !cell.revealed {
            return queue!(out, Print("⚑ ".with(Color::Yellow)));
        }
        if cell.mine && cell.revealed {
            return queue!(out, Print("✸ ".with(Color::Red)));
        }
        if !cell.revealed {
            return queue!(out, Print("■ ".with(Color::DarkGrey)));
        }
        if cell.adjacent == 0 {
            return queue!(out, Print("  "));
        }
        let color = number_color(cell.adjacent);
        queue!(out, Print(format!("{} ", cell.adjacent).with(color)))
    }
}

fn number_color(n: u8) -> Color {
    match n {
        1 => Color::Blue,
        2 => Color::Green,
        3 => Color::Red,
        4 => Color::Cyan,
        5 => Color::Magenta,
        6 => Color::Yellow,
        7 => Color::White,
        _ => Color::DarkGrey,
    }
}

fn mmss(d: Duration) -> String {
    let total = d.as_secs();
    format!("{:02}:{:02}", total / 60, total % 60)
}

/// Runs one minesweeper session (until the user quits back to the menu).
pub fn run(diff: &Difficulty) -> io::Result<()> {
    let mut out = io::stdout();
    let mut game = Game::new(diff);
    let mut last_time = String::new();
    let mut dirty = true;
    loop {
        let now = mmss(game.elapsed());
        if dirty || now != last_time {
            game.render(&mut out, diff)?;
            last_time = now;
            dirty = false;
        }
        if let Some(Event::Key(key)) = term::poll_event(term::POLL_MS)? {
            match game.handle_key(key) {
                Action::Continue => {}
                Action::Restart => {
                    game = Game::new(diff);
                    last_time.clear();
                }
                Action::Quit => return Ok(()),
            }
            dirty = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_to_string(game: &Game, diff: &Difficulty) -> String {
        let mut buf = Vec::new();
        game.render(&mut buf, diff).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn frame_contains_header_and_footer() {
        let diff = &DIFFICULTIES[0];
        let game = Game::new(diff);
        let frame = render_to_string(&game, diff);
        assert!(frame.contains("DÒ MÌN"));
        assert!(frame.contains("Mìn:"));
        assert!(frame.contains("Thời gian:"));
        assert!(frame.contains("X đào"));
    }

    #[test]
    fn revealed_numbers_appear_in_frame() {
        let diff = &DIFFICULTIES[0];
        let mut game = Game::new(diff);
        // Dig every cell except mines → some number cells must be visible.
        for r in 0..diff.rows {
            for c in 0..diff.cols {
                if !game.board.cell(r, c).mine {
                    game.board.dig(r, c);
                }
            }
        }
        let frame = render_to_string(&game, diff);
        assert!(frame.contains('1'), "expected at least one number cell");
        assert!(frame.contains("BẠN THẮNG"), "revealing all safe cells must win");
    }

    #[test]
    fn lost_frame_shows_defeat_message() {
        let diff = &DIFFICULTIES[0];
        let mut game = Game::new(diff);
        // First dig is safe and triggers mine placement…
        game.board.dig(0, 0);
        // …now find an (unrevealed) mine and boom into it.
        let mine_at = (0..diff.rows)
            .flat_map(|r| (0..diff.cols).map(move |c| (r, c)))
            .find(|&(r, c)| game.board.cell(r, c).mine && !game.board.cell(r, c).revealed)
            .unwrap();
        game.cursor = mine_at;
        game.dig();
        let frame = render_to_string(&game, diff);
        assert!(frame.contains("BẠN THUA"));
    }

    #[test]
    fn mmss_formats_as_mm_ss() {
        assert_eq!(mmss(Duration::from_secs(65)), "01:05");
        assert_eq!(mmss(Duration::from_secs(0)), "00:00");
    }
}
