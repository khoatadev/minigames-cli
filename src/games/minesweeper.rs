//! Minesweeper — terminal UI and game loop.
//!
//! Controls: arrow keys move the cursor, `X` digs, `Z` toggles a flag,
//! `R` restarts, `Esc` returns to the menu.

use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::cursor::MoveTo;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::queue;
use crossterm::style::{Color, Print, SetBackgroundColor, SetForegroundColor, Stylize};

use crate::board::{Board, ChordOutcome, DigResult, Status};
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
    /// The mine cell that was dug when the game was lost — drawn bright red.
    exploded: Option<(usize, usize)>,
}

impl Game {
    fn new(diff: &Difficulty) -> Self {
        Self {
            board: Board::new(diff.rows, diff.cols, diff.mines),
            cursor: (0, 0),
            started: Instant::now(),
            ended: None,
            exploded: None,
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
                KeyCode::Esc => Action::Quit,
                _ => Action::Continue,
            };
        }
        let (r, c) = self.cursor;
        match key.code {
            KeyCode::Left if c > 0 => self.cursor.1 -= 1,
            KeyCode::Right if c + 1 < self.board.cols => self.cursor.1 += 1,
            KeyCode::Up if r > 0 => self.cursor.0 -= 1,
            KeyCode::Down if r + 1 < self.board.rows => self.cursor.0 += 1,
            KeyCode::Char('x' | 'X') => {
                let cell = self.board.cell(r, c);
                if cell.revealed && cell.adjacent > 0 {
                    self.chord();
                } else {
                    self.dig();
                }
            }
            KeyCode::Char('z' | 'Z') => {
                self.board.toggle_flag(r, c);
            }
            KeyCode::Char('r' | 'R') => return Action::Restart,
            KeyCode::Esc => return Action::Quit,
            _ => {}
        }
        Action::Continue
    }

    fn dig(&mut self) {
        let (r, c) = self.cursor;
        match self.board.dig(r, c) {
            DigResult::Boomed => {
                self.exploded = self.board.first_revealed_mine();
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

    /// Chording on a revealed number cell: opens the remaining neighbours
    /// when the surrounding flag count matches the number.
    fn chord(&mut self) {
        let (r, c) = self.cursor;
        match self.board.chord(r, c) {
            ChordOutcome::Boomed => {
                self.exploded = self.board.first_revealed_mine();
                self.board.reveal_all_mines();
                self.ended = Some(Instant::now());
            }
            ChordOutcome::Revealed if self.board.status() == Status::Won => {
                self.board.auto_flag_remaining();
                self.ended = Some(Instant::now());
            }
            _ => {}
        }
    }

    /// Draws one frame into `out`. Terminal dimensions are parameters so
    /// rendering is testable without a TTY; the game loop passes real ones.
    fn render(
        &self,
        out: &mut impl Write,
        diff: &Difficulty,
        term_cols: u16,
        term_rows: u16,
    ) -> io::Result<()> {
        term::begin_frame(out)?;
        // Board (incl. its frame) is centered; header and footer are centered
        // on the same vertical axis so everything lines up in the middle.
        const FRAME_PAD: u16 = 2; // │ borders on both sides
        let frame_w = self.board.cols as u16 * 2 + FRAME_PAD;
        let frame_h = self.board.rows as u16 + 2; // top + bottom border
        let frame_total = frame_h + 4; // header, blank, frame, blank, footer
        let (x0, y0) = term::center_offset(term_cols, term_rows, frame_w, frame_total);
        let mines_left = self.board.mines as isize - self.board.flags_placed() as isize;

        // Header
        let mines_color = match mines_left {
            n if n < 0 => Color::Red,
            0 => Color::Green,
            _ => Color::Yellow,
        };
        let header = vec![
            (format!("DÒ MÌN - {}   ", diff.name), Color::Reset),
            ("Mìn: ".to_string(), Color::DarkGrey),
            (format!("{mines_left}   "), mines_color),
            ("Thời gian: ".to_string(), Color::DarkGrey),
            (term::mmss(self.elapsed()), Color::Reset),
        ];
        print_centered(out, y0, &header, term_cols)?;

        // Frame around the board — muted grey so it doesn't fight the tiles
        let bar = "─".repeat(self.board.cols * 2);
        queue!(out, MoveTo(x0, y0 + 2), Print(format!("┌{bar}┐").with(Color::DarkGrey)))?;
        queue!(
            out,
            MoveTo(x0, y0 + 2 + self.board.rows as u16 + 1),
            Print(format!("└{bar}┘").with(Color::DarkGrey))
        )?;

        // Board cells, flanked by │ borders
        for r in 0..self.board.rows {
            queue!(out, MoveTo(x0, y0 + 3 + r as u16), Print("│".with(Color::DarkGrey)))?;
            for c in 0..self.board.cols {
                self.draw_cell(out, r, c)?;
            }
            queue!(out, Print("│".with(Color::DarkGrey)))?;
        }

        // Status / footer
        let status_row = y0 + 2 + self.board.rows as u16 + 2;
        let footer: Vec<(String, Color)> = match self.board.status() {
            Status::Lost => vec![
                ("BẠN THUA!  ".to_string(), Color::Red),
                ("R chơi lại, Esc về menu".to_string(), Color::DarkGrey),
            ],
            Status::Won => vec![
                (
                    format!("BẠN THẮNG! Thời gian: {}   ", term::mmss(self.elapsed())),
                    Color::Green,
                ),
                ("R chơi lại, Esc về menu".to_string(), Color::DarkGrey),
            ],
            Status::InProgress => vec![
                ("X".to_string(), Color::Yellow),
                (" đào   ".to_string(), Color::DarkGrey),
                ("Z".to_string(), Color::Yellow),
                (" cắm/gỡ cờ   ".to_string(), Color::DarkGrey),
                ("R".to_string(), Color::Yellow),
                (" chơi lại   ".to_string(), Color::DarkGrey),
                ("Esc".to_string(), Color::Yellow),
                (" về menu".to_string(), Color::DarkGrey),
            ],
        };
        print_centered(out, status_row, &footer, term_cols)?;
        out.flush()
    }

    /// Draws one cell as `glyph + gap` — a coloured glyph followed by a
    /// plain space, like the original style. Only the cursor (white
    /// background) and the exploded mine (solid red block) use background
    /// colours: background fills the whole cell height, so per-cell
    /// backgrounds on adjacent rows would merge into stripes.
    fn draw_cell(&self, out: &mut impl Write, r: usize, c: usize) -> io::Result<()> {
        let cell = self.board.cell(r, c);
        let on_cursor = self.cursor == (r, c);
        let exploded = self.exploded == Some((r, c));

        let (glyph, fg, bg) = if exploded {
            ("✸".to_string(), Color::Black, Color::Red)
        } else if on_cursor {
            let glyph = if cell.flagged && !cell.revealed {
                "⚑".to_string()
            } else if cell.revealed && cell.mine {
                "✸".to_string()
            } else if cell.revealed && cell.adjacent > 0 {
                format!("{}", cell.adjacent)
            } else if cell.revealed {
                " ".to_string()
            } else {
                "■".to_string()
            };
            (glyph, Color::Black, Color::White)
        } else if cell.revealed && cell.mine {
            ("✸".to_string(), Color::Red, Color::Reset)
        } else if cell.flagged && !cell.revealed {
            ("⚑".to_string(), Color::Yellow, Color::Reset)
        } else if !cell.revealed {
            ("■".to_string(), Color::DarkGrey, Color::Reset)
        } else if cell.adjacent == 0 {
            (" ".to_string(), Color::Reset, Color::Reset)
        } else {
            (format!("{}", cell.adjacent), number_color(cell.adjacent), Color::Reset)
        };

        queue!(
            out,
            SetBackgroundColor(bg),
            SetForegroundColor(fg),
            Print(glyph),
            SetBackgroundColor(Color::Reset),
            SetForegroundColor(Color::Reset),
            Print(" ")
        )
    }
}

/// Prints styled text segments on one row, centered horizontally in a
/// `cols`-wide terminal. Width is measured in visible characters.
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

/// Runs one minesweeper session (until the user quits back to the menu).
pub fn run(diff: &Difficulty) -> io::Result<()> {
    let mut out = io::stdout();
    let mut game = Game::new(diff);
    let mut last_time = String::new();
    let mut dirty = true;
    loop {
        let now = term::mmss(game.elapsed());
        if dirty || now != last_time {
            let (cols, rows) = term::dims()?;
            game.render(&mut out, diff, cols, rows)?;
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
        game.render(&mut buf, diff, 80, 24).unwrap();
        strip_ansi(String::from_utf8(buf).unwrap())
    }

    /// Removes CSI escape sequences so assertions see plain text.
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
    fn x_on_a_number_cell_with_matching_flags_chords() {
        let diff = &DIFFICULTIES[0];
        let mut game = Game::new(diff);
        game.board.plant_mines(&[(0, 0), (0, 2)]);
        game.board.dig(1, 1);
        assert_eq!(game.board.cell(1, 1).adjacent, 2);
        game.board.toggle_flag(0, 0);
        game.board.toggle_flag(0, 2);
        game.cursor = (1, 1);
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
        game.handle_key(key);
        assert!(game.board.cell(1, 0).revealed, "chord must open neighbours");
        assert!(game.board.cell(2, 2).revealed);
        assert!(!game.board.cell(0, 0).revealed, "flagged mines stay hidden");
    }

    #[test]
    fn x_on_an_unrevealed_cell_still_digs_normally() {
        let diff = &DIFFICULTIES[0];
        let mut game = Game::new(diff);
        game.board.plant_mines(&[(0, 0), (0, 2)]);
        game.cursor = (1, 1);
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
        game.handle_key(key);
        assert!(game.board.cell(1, 1).revealed);
        assert_eq!(game.board.status(), Status::InProgress);
    }

    #[test]
    fn frame_draws_muted_border_and_blue_cells() {
        let diff = &DIFFICULTIES[0];
        let game = Game::new(diff);
        let mut buf = Vec::new();
        game.render(&mut buf, diff, 80, 24).unwrap();
        let raw = String::from_utf8(buf).unwrap();
        assert!(raw.contains('┌') && raw.contains('└') && raw.contains('│'), "border");
        // Hidden cells are muted grey ■ glyphs on the default background —
        // bright colours are reserved for numbers, flags and mines.
        assert!(raw.contains("\x1b[38;5;8m■"), "hidden cells must be grey glyphs");
        assert!(!raw.contains("\x1b[38;5;12m■"), "no bright blue hidden cells");
        assert!(!raw.contains("\x1b[48;5;4m"), "no blue background tiles");
        // The cursor at (0,0) is a white block.
        assert!(raw.contains("\x1b[48;5;15m"), "cursor must be a white tile");
        assert!(!raw.contains("\x1b[38;5;14m"), "cyan must be gone from the board");
    }

    #[test]
    fn lost_frame_paints_mines_red() {
        let diff = &DIFFICULTIES[0];
        let mut game = Game::new(diff);
        game.board.dig(0, 0);
        let mine_at = (0..diff.rows)
            .flat_map(|r| (0..diff.cols).map(move |c| (r, c)))
            .find(|&(r, c)| game.board.cell(r, c).mine && !game.board.cell(r, c).revealed)
            .unwrap();
        game.cursor = mine_at;
        game.dig();
        let mut buf = Vec::new();
        game.render(&mut buf, diff, 80, 24).unwrap();
        let raw = String::from_utf8(buf).unwrap();
        assert!(raw.contains("\x1b[48;5;9m"), "exploded mine must be a solid red block");
        assert!(raw.contains("\x1b[38;5;9m✸"), "other mines must be red ✸ glyphs");
    }
}
