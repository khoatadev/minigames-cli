//! Main menu — the hub that lists all mini games.

use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::queue;
use crossterm::style::{Color, Print, Stylize};

use crate::games::minesweeper::{self, DIFFICULTIES};
use crate::games::reaction;
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

const GAMES: [(&str, bool); 4] = [
    ("Dò mìn", true),
    ("Phản xạ", true),
    ("Caro", false),
    ("Rắn săn mồi", false),
];

const BANNER: [&str; 4] = [
    r#" __  __ ___ _  _ ___   ___   _   __  __ ___ ___ "#,
    r#"|  \/  |_ _| \| |_ _| / __| /_\ |  \/  | __/ __|"#,
    r#"| |\/| || || .` || | | (_ |/ _ \| |\/| | _|\__ \"#,
    r#"|_|  |_|___|_|\_|___| \___/_/ \_\_|  |_|___|___/"#,
];
const BANNER_W: u16 = 48;

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
            Some(1) => reaction::run()?,
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
    let mut dirty = true;
    loop {
        if dirty {
            let (cols, rows) = term::dims()?;
            render_list(out, title, entries, sel, footer, cols, rows)?;
            dirty = false;
        }
        if let Some(Event::Key(key)) = term::poll_event(term::POLL_MS)? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            dirty = true;
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
    cols: u16,
    rows: u16,
) -> io::Result<()> {
    term::begin_frame(out)?;

    let show_banner = title == "MINI GAMES" && cols >= BANNER_W + 4 && rows >= 20;
    let banner_h = if show_banner { BANNER.len() as u16 + 1 } else { 0 };

    // Box dimensions
    let box_w: u16 = 50.min(cols.saturating_sub(4)).max(34);
    let inner_w = (box_w.saturating_sub(2)) as usize;
    let box_h: u16 = entries.len() as u16 + 7;
    let total_h = banner_h + box_h;

    let start_y = rows.saturating_sub(total_h) / 2;
    let mut cur_y = start_y;

    // Draw ASCII banner if conditions met
    if show_banner {
        let banner_x = cols.saturating_sub(BANNER_W) / 2;
        for line in &BANNER {
            queue!(out, MoveTo(banner_x, cur_y), Print(line.with(Color::Cyan).bold()))?;
            cur_y += 1;
        }
        cur_y += 1; // blank separator
    }

    let box_x = cols.saturating_sub(box_w) / 2;
    let bar = "─".repeat(inner_w);

    // Box top border
    queue!(out, MoveTo(box_x, cur_y), Print(format!("╭{bar}╮").with(Color::DarkGrey)))?;
    cur_y += 1;

    // Box Title
    let title_text = format!("★ {} ★", title);
    let title_vis_len = title_text.chars().count();
    let title_lpad = inner_w.saturating_sub(title_vis_len) / 2;
    let title_rpad = inner_w.saturating_sub(title_vis_len + title_lpad);
    queue!(
        out,
        MoveTo(box_x, cur_y),
        Print("│".with(Color::DarkGrey)),
        Print(" ".repeat(title_lpad)),
        Print(title_text.with(Color::Yellow).bold()),
        Print(" ".repeat(title_rpad)),
        Print("│".with(Color::DarkGrey))
    )?;
    cur_y += 1;

    // Header divider
    queue!(out, MoveTo(box_x, cur_y), Print(format!("├{bar}┤").with(Color::DarkGrey)))?;
    cur_y += 1;

    // Blank row inside box
    queue!(
        out,
        MoveTo(box_x, cur_y),
        Print("│".with(Color::DarkGrey)),
        Print(" ".repeat(inner_w)),
        Print("│".with(Color::DarkGrey))
    )?;
    cur_y += 1;

    // Entries
    for (i, entry) in entries.iter().enumerate() {
        let marker = if i == sel { "▶ " } else { "  " };
        let disabled_tag = if !entry.enabled { "   (sắp ra mắt)" } else { "" };
        let vis_len = 2 + 2 + entry.label.chars().count() + disabled_tag.chars().count();
        let rpad = inner_w.saturating_sub(vis_len);

        queue!(
            out,
            MoveTo(box_x, cur_y),
            Print("│".with(Color::DarkGrey)),
            Print("  ")
        )?;

        if i == sel {
            queue!(
                out,
                Print(marker.with(Color::Yellow).bold()),
                Print(entry.label.clone().with(Color::White).bold())
            )?;
        } else {
            let color = if entry.enabled { Color::Reset } else { Color::DarkGrey };
            queue!(
                out,
                Print(marker.with(Color::DarkGrey)),
                Print(entry.label.clone().with(color))
            )?;
        }

        if !entry.enabled {
            queue!(out, Print(disabled_tag.with(Color::DarkGrey)))?;
        }

        queue!(
            out,
            Print(" ".repeat(rpad)),
            Print("│".with(Color::DarkGrey))
        )?;
        cur_y += 1;
    }

    // Blank row inside box
    queue!(
        out,
        MoveTo(box_x, cur_y),
        Print("│".with(Color::DarkGrey)),
        Print(" ".repeat(inner_w)),
        Print("│".with(Color::DarkGrey))
    )?;
    cur_y += 1;

    // Footer divider
    queue!(out, MoveTo(box_x, cur_y), Print(format!("├{bar}┤").with(Color::DarkGrey)))?;
    cur_y += 1;

    // Footer text
    let footer_vis_len = footer.chars().count();
    let footer_lpad = inner_w.saturating_sub(footer_vis_len) / 2;
    let footer_rpad = inner_w.saturating_sub(footer_vis_len + footer_lpad);
    queue!(
        out,
        MoveTo(box_x, cur_y),
        Print("│".with(Color::DarkGrey)),
        Print(" ".repeat(footer_lpad)),
        Print(footer.with(Color::DarkGrey)),
        Print(" ".repeat(footer_rpad)),
        Print("│".with(Color::DarkGrey))
    )?;
    cur_y += 1;

    // Box bottom border
    queue!(out, MoveTo(box_x, cur_y), Print(format!("╰{bar}╯").with(Color::DarkGrey)))?;

    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(entries: &[MenuEntry], sel: usize) -> String {
        let mut buf = Vec::new();
        render_list(&mut buf, "MINI GAMES", entries, sel, "footer", 80, 24).unwrap();
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
        assert!(render(&entries, 1).contains("▶ Phản xạ"));
        assert!(render(&entries, 2).contains("▶ Caro"));
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
