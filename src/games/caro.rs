//! Caro (Gomoku) — terminal UI and game loop.
//!
//! Controls: arrow keys move cursor, Enter/Space/X places a stone,
//! R restarts the match, Esc returns to the menu.

use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::cursor::MoveTo;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::queue;
use crossterm::style::{Color, Print, SetBackgroundColor, SetForegroundColor, Stylize};

use crate::caro::{BOARD_SIZE, CaroBoard, GameStatus, PlaceResult, Player};
use crate::term;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    PvE, // Player (X) vs Bot AI (O)
    PvP, // 2 Players on same keyboard
}

impl GameMode {
    pub fn name(self) -> &'static str {
        match self {
            GameMode::PvE => "Đấu với Máy",
            GameMode::PvP => "2 Người chơi",
        }
    }
}

enum Action {
    Continue,
    Restart,
    Quit,
}

pub struct CaroGame {
    pub board: CaroBoard,
    pub mode: GameMode,
    pub cursor: (usize, usize),
    pub started: Instant,
    pub ended: Option<Instant>,
}

impl CaroGame {
    pub fn new(mode: GameMode) -> Self {
        Self {
            board: CaroBoard::new(),
            mode,
            cursor: (BOARD_SIZE / 2, BOARD_SIZE / 2),
            started: Instant::now(),
            ended: None,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.ended.unwrap_or_else(Instant::now) - self.started
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.kind != KeyEventKind::Press {
            return Action::Continue;
        }

        if self.board.status() != GameStatus::InProgress {
            return match key.code {
                KeyCode::Char('r' | 'R') => Action::Restart,
                KeyCode::Esc => Action::Quit,
                _ => Action::Continue,
            };
        }

        let (r, c) = self.cursor;
        match key.code {
            KeyCode::Left | KeyCode::Char('a' | 'A') if c > 0 => self.cursor.1 -= 1,
            KeyCode::Right | KeyCode::Char('d' | 'D') if c + 1 < BOARD_SIZE => self.cursor.1 += 1,
            KeyCode::Up | KeyCode::Char('w' | 'W') if r > 0 => self.cursor.0 -= 1,
            KeyCode::Down | KeyCode::Char('s' | 'S') if r + 1 < BOARD_SIZE => self.cursor.0 += 1,
            KeyCode::Enter | KeyCode::Char(' ' | 'x' | 'X') => {
                if self.board.place(r, c) == PlaceResult::Success
                    && self.board.status() != GameStatus::InProgress
                {
                    self.ended = Some(Instant::now());
                }
            }
            KeyCode::Char('r' | 'R') => return Action::Restart,
            KeyCode::Esc => return Action::Quit,
            _ => {}
        }

        Action::Continue
    }

    fn maybe_bot_move(&mut self) -> bool {
        if self.mode == GameMode::PvE
            && self.board.turn() == Player::O
            && self.board.status() == GameStatus::InProgress
            && let Some((r, c)) = self.board.find_best_move()
        {
            self.board.place(r, c);
            if self.board.status() != GameStatus::InProgress {
                self.ended = Some(Instant::now());
            }
            return true;
        }
        false
    }

    pub fn render(
        &self,
        out: &mut impl Write,
        term_cols: u16,
        term_rows: u16,
    ) -> io::Result<()> {
        term::begin_frame(out)?;

        // Board dimensions: 15 cols * 2 chars + 2 border chars = 32
        const BOARD_W: u16 = (BOARD_SIZE as u16) * 2 + 2;
        const BOARD_H: u16 = (BOARD_SIZE as u16) + 2;
        const TOTAL_H: u16 = BOARD_H + 4; // header, blank, board, blank, footer

        let (x0, y0) = term::center_offset(term_cols, term_rows, BOARD_W, TOTAL_H);

        // Header
        let turn_badge = match self.board.status() {
            GameStatus::InProgress => format!("Lượt: [{}]   ", self.board.turn().symbol()),
            _ => String::new(),
        };

        let header = vec![
            (format!("★ CỜ CARO - {} ★   ", self.mode.name()), Color::Cyan),
            (turn_badge, Color::Yellow),
            ("Nước: ".to_string(), Color::DarkGrey),
            (format!("{}   ", self.board.moves_count()), Color::White),
            ("Thời gian: ".to_string(), Color::DarkGrey),
            (term::mmss(self.elapsed()), Color::Reset),
        ];
        print_centered(out, y0, &header, term_cols)?;

        // Board top border
        let bar = "─".repeat(BOARD_SIZE * 2);
        queue!(out, MoveTo(x0, y0 + 2), Print(format!("┌{bar}┐").with(Color::DarkGrey)))?;

        // Board cells
        for r in 0..BOARD_SIZE {
            queue!(out, MoveTo(x0, y0 + 3 + r as u16), Print("│".with(Color::DarkGrey)))?;
            for c in 0..BOARD_SIZE {
                self.draw_cell(out, r, c)?;
            }
            queue!(out, Print("│".with(Color::DarkGrey)))?;
        }

        // Board bottom border
        queue!(
            out,
            MoveTo(x0, y0 + 3 + BOARD_SIZE as u16),
            Print(format!("└{bar}┘").with(Color::DarkGrey))
        )?;

        // Footer / status
        let footer_row = y0 + 3 + BOARD_SIZE as u16 + 2;
        let footer = match self.board.status() {
            GameStatus::Won(p) => {
                let win_msg = match self.mode {
                    GameMode::PvE if p == Player::X => "🎉 BẠN THẮNG! 🎉   ".to_string(),
                    GameMode::PvE => "🤖 MÁY THẮNG! 🤖   ".to_string(),
                    GameMode::PvP => format!("🎉 NGƯỜI CHƠI {} THẮNG! 🎉   ", p.symbol()),
                };
                let win_color = if self.mode == GameMode::PvE && p == Player::O {
                    Color::Red
                } else {
                    Color::Green
                };
                vec![
                    (win_msg, win_color),
                    ("[R]".to_string(), Color::Yellow),
                    (" chơi lại   ".to_string(), Color::DarkGrey),
                    ("[Esc]".to_string(), Color::Yellow),
                    (" về menu".to_string(), Color::DarkGrey),
                ]
            }
            GameStatus::Draw => vec![
                ("🤝 BÀN CỜ HÒA! 🤝   ".to_string(), Color::Yellow),
                ("[R]".to_string(), Color::Yellow),
                (" chơi lại   ".to_string(), Color::DarkGrey),
                ("[Esc]".to_string(), Color::Yellow),
                (" về menu".to_string(), Color::DarkGrey),
            ],
            GameStatus::InProgress => vec![
                ("↑↓←→".to_string(), Color::Yellow),
                (" di chuyển   ".to_string(), Color::DarkGrey),
                ("Enter/Space/X".to_string(), Color::Yellow),
                (" đánh   ".to_string(), Color::DarkGrey),
                ("R".to_string(), Color::Yellow),
                (" ván mới   ".to_string(), Color::DarkGrey),
                ("Esc".to_string(), Color::Yellow),
                (" menu".to_string(), Color::DarkGrey),
            ],
        };
        print_centered(out, footer_row, &footer, term_cols)?;

        out.flush()
    }

    fn draw_cell(&self, out: &mut impl Write, r: usize, c: usize) -> io::Result<()> {
        let on_cursor = self.cursor == (r, c);
        let cell = self.board.cell(r, c);
        let is_win = self
            .board
            .winning_line()
            .is_some_and(|line| line.contains(&(r, c)));
        let is_last = self.board.last_move() == Some((r, c));

        let (glyph, default_fg) = match cell {
            Some(Player::X) => ("✕ ", Color::Cyan),
            Some(Player::O) => ("◯ ", Color::Yellow),
            None => ("· ", Color::DarkGrey),
        };

        let (fg, bg) = if on_cursor {
            (Color::Black, Color::White)
        } else if is_win {
            (Color::White, Color::Green)
        } else if is_last {
            (Color::White, Color::DarkGrey)
        } else {
            (default_fg, Color::Reset)
        };

        queue!(
            out,
            SetBackgroundColor(bg),
            SetForegroundColor(fg),
            Print(glyph),
            SetBackgroundColor(Color::Reset),
            SetForegroundColor(Color::Reset),
        )
    }
}

/// Runs the Caro game loop for the selected mode.
pub fn run(mode: GameMode) -> io::Result<()> {
    let mut out = io::stdout();
    loop {
        let mut game = CaroGame::new(mode);
        let mut dirty = true;
        let mut last_second = u64::MAX;

        loop {
            // Check for bot move if PvE
            if game.maybe_bot_move() {
                dirty = true;
            }

            let elapsed_sec = game.elapsed().as_secs();
            if dirty || elapsed_sec != last_second {
                let (cols, rows) = term::dims()?;
                game.render(&mut out, cols, rows)?;
                last_second = elapsed_sec;
                dirty = false;
            }

            if let Some(Event::Key(key)) = term::poll_event(term::POLL_MS)? {
                match game.handle_key(key) {
                    Action::Restart => break,
                    Action::Quit => return Ok(()),
                    Action::Continue => dirty = true,
                }
            }
        }
    }
}

fn print_centered(
    out: &mut impl Write,
    row: u16,
    parts: &[(String, Color)],
    cols: u16,
) -> io::Result<()> {
    let width: usize = parts.iter().map(|(s, _)| s.chars().count()).sum();
    queue!(out, MoveTo(cols.saturating_sub(width as u16) / 2, row))?;
    for (text, color) in parts {
        queue!(out, Print(text.clone().with(*color)))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip_ansi(s: String) -> String {
        let mut out = String::with_capacity(s.len());
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                for esc in chars.by_ref() {
                    if esc.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    #[test]
    fn render_in_progress_frame_shows_header_and_controls() {
        let game = CaroGame::new(GameMode::PvE);
        let mut buf = Vec::new();
        game.render(&mut buf, 80, 24).unwrap();
        let plain = strip_ansi(String::from_utf8(buf).unwrap());
        assert!(plain.contains("CỜ CARO"));
        assert!(plain.contains("Đấu với Máy"));
        assert!(plain.contains("di chuyển"));
        assert!(plain.contains("đánh"));
        assert!(plain.contains('┌') && plain.contains('└') && plain.contains('│'));
    }

    #[test]
    fn render_won_frame_shows_win_message() {
        let mut game = CaroGame::new(GameMode::PvP);
        for c in 0..5 {
            game.board.place(7, c); // X
            if c < 4 {
                game.board.place(8, c); // O
            }
        }
        assert_eq!(game.board.status(), GameStatus::Won(Player::X));
        let mut buf = Vec::new();
        game.render(&mut buf, 80, 24).unwrap();
        let plain = strip_ansi(String::from_utf8(buf).unwrap());
        assert!(plain.contains("NGƯỜI CHƠI X THẮNG"));
        assert!(plain.contains("chơi lại"));
    }
}
