//! Main menu — the hub that lists all mini games.

use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::queue;
use crossterm::style::{Color, Print, SetBackgroundColor, Stylize};

use crate::games::minesweeper::{self, DIFFICULTIES};
use crate::term;

struct MenuEntry {
    label: String,
    enabled: bool,
}

impl MenuEntry {
    fn new(label: impl Into<String>, enabled: bool) -> Self {
        Self { label: label.into(), enabled }
    }
}

const GAMES: [(&str, bool); 3] = [
    ("Dò mìn", true),
    ("Caro", false),
    ("Rắn săn mồi", false),
];

pub fn run() -> io::Result<()> {
    let mut out = io::stdout();
    loop {
        let games: Vec<MenuEntry> =
            GAMES.iter().map(|&(label, on)| MenuEntry::new(label, on)).collect();
        match select(&mut out, "MINI GAMES", &games, "↑↓ chọn   Enter chơi   Q thoát")? {
            Some(0) => {
                let diffs: Vec<MenuEntry> = DIFFICULTIES
                    .iter()
                    .map(|d| MenuEntry::new(format!("{}  ({}x{}, {} mìn)", d.name, d.cols, d.rows, d.mines), true))
                    .chain(std::iter::once(MenuEntry::new("← Về menu", true)))
                    .collect();
                match select(
                    &mut out,
                    "DÒ MÌN — chọn độ khó",
                    &diffs,
                    "↑↓ chọn   Enter chơi   Esc về menu",
                )? {
                    Some(i) if i < DIFFICULTIES.len() => minesweeper::run(&DIFFICULTIES[i])?,
                    _ => continue,
                }
            }
            _ => return Ok(()),
        }
    }
}

/// Renders a selectable list and returns the highlighted index on Enter,
/// or `None` on quit/back. Enter on a disabled entry is ignored.
fn select(
    out: &mut impl Write,
    title: &str,
    entries: &[MenuEntry],
    footer: &str,
) -> io::Result<Option<usize>> {
    let mut sel = 0usize;
    loop {
        render_list(out, title, entries, sel, footer)?;
        if let Some(Event::Key(key)) = term::poll_event(term::POLL_MS)? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Up => sel = (sel + entries.len() - 1) % entries.len(),
                KeyCode::Down => sel = (sel + 1) % entries.len(),
                KeyCode::Enter => {
                    if entries[sel].enabled {
                        return Ok(Some(sel));
                    }
                }
                KeyCode::Esc | KeyCode::Char('q' | 'Q') => return Ok(None),
                _ => {}
            }
        }
    }
}

fn render_list(
    out: &mut impl Write,
    title: &str,
    entries: &[MenuEntry],
    sel: usize,
    footer: &str,
) -> io::Result<()> {
    term::begin_frame(out)?;
    queue!(out, MoveTo(2, 0), Print(title.with(Color::Cyan)))?;
    for (i, entry) in entries.iter().enumerate() {
        queue!(out, MoveTo(2, (i + 2) as u16))?;
        let marker = if i == sel { "▶ " } else { "  " };
        if i == sel {
            queue!(out, SetBackgroundColor(Color::DarkGrey))?;
        }
        let color = if entry.enabled { Color::Reset } else { Color::DarkGrey };
        queue!(out, Print(marker), Print(entry.label.clone().with(color)))?;
        if i == sel {
            queue!(out, SetBackgroundColor(Color::Reset))?;
        }
        if !entry.enabled {
            queue!(out, Print("   (sắp ra mắt)".with(Color::DarkGrey)))?;
        }
    }
    let footer_row = (entries.len() + 3) as u16;
    queue!(out, MoveTo(2, footer_row), Print(footer.with(Color::DarkGrey)))?;
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(entries: &[MenuEntry], sel: usize) -> String {
        let mut buf = Vec::new();
        render_list(&mut buf, "MINI GAMES", entries, sel, "footer").unwrap();
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
    fn menu_lists_all_games() {
        let entries: Vec<MenuEntry> =
            GAMES.iter().map(|&(l, on)| MenuEntry::new(l, on)).collect();
        let frame = render(&entries, 0);
        assert!(frame.contains("MINI GAMES"));
        assert!(frame.contains("Dò mìn"));
        assert!(frame.contains("Caro"));
        assert!(frame.contains("sắp ra mắt"));
    }

    #[test]
    fn selection_marker_moves_with_index() {
        let entries: Vec<MenuEntry> =
            GAMES.iter().map(|&(l, on)| MenuEntry::new(l, on)).collect();
        assert!(render(&entries, 1).contains("▶ Caro"));
    }

    #[test]
    fn difficulty_menu_shows_sizes_and_back_row() {
        let diffs: Vec<MenuEntry> = DIFFICULTIES
            .iter()
            .map(|d| MenuEntry::new(format!("{}  ({}x{}, {} mìn)", d.name, d.cols, d.rows, d.mines), true))
            .chain(std::iter::once(MenuEntry::new("← Về menu", true)))
            .collect();
        let frame = render(&diffs, 0);
        assert!(frame.contains("9x9, 10 mìn"));
        assert!(frame.contains("30x16, 99 mìn"));
        assert!(frame.contains("← Về menu"));
    }
}
