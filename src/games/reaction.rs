//! Reaction game — terminal UI and game loop.
//!
//! A letter appears somewhere on screen; type it as fast as you can. Faster
//! hits score more, consecutive hits build a combo bonus, wrong keys cost
//! points. A round lasts 60 seconds; `Esc` ends it early. After the round a
//! summary shows the score and stats, with `R` to replay and `Esc` for the
//! menu.

use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::cursor::MoveTo;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::queue;
use crossterm::style::{Color, Print, Stylize};
use rand::RngExt;

use crate::reaction::{PressResult, Session};
use crate::term;

/// True when the key is a plain letter press (Shift for capitals is fine,
/// Ctrl/Alt combos are ignored rather than punished).
fn is_plain_key(key: &KeyEvent) -> bool {
    matches!(key.modifiers, KeyModifiers::NONE | KeyModifiers::SHIFT)
}

/// Palette the targets cycle through; indexed by `Target::color`.
const PALETTE: [Color; 7] = [
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::White,
];

enum Outcome {
    Replay,
    Quit,
}

pub fn run() -> io::Result<()> {
    let mut out = io::stdout();
    loop {
        match play_round(&mut out)? {
            Outcome::Replay => continue,
            Outcome::Quit => return Ok(()),
        }
    }
}

/// Plays one 60-second round and its summary, then asks to replay or quit.
fn play_round(out: &mut impl Write) -> io::Result<Outcome> {
    let t0 = Instant::now();
    let mut s = Session::new(t0);
    spawn_next(&mut s, None)?;
    let mut last_char = s.target().map(|t| t.ch);

    let mut dirty = true;
    let mut last_second = u64::MAX;
    let mut ended_early = false;

    loop {
        let now = Instant::now();
        if s.finished(now) {
            break;
        }
        let left = s.time_left(now);
        let second = left.as_secs();
        if dirty || second != last_second {
            render_play(out, &s, left)?;
            last_second = second;
            dirty = false;
        }
        if let Some(crossterm::event::Event::Key(key)) = term::poll_event(term::POLL_MS)? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Esc => {
                    ended_early = true;
                    break;
                }
                KeyCode::Char(c) if is_plain_key(&key) => match s.press(c, now) {
                    PressResult::Hit { .. } => {
                        spawn_next(&mut s, last_char)?;
                        last_char = s.target().map(|t| t.ch);
                        dirty = true;
                    }
                    PressResult::Wrong => dirty = true,
                    PressResult::Ignored => {}
                },
                _ => {}
            }
        }
    }

    render_summary(out, &s, ended_early)?;
    loop {
        if let Some(crossterm::event::Event::Key(key)) = term::poll_event(term::POLL_MS)? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('r' | 'R') => return Ok(Outcome::Replay),
                KeyCode::Esc => return Ok(Outcome::Quit),
                _ => {}
            }
        }
    }
}

/// Spawns a random letter in a random palette colour at a random position,
/// avoiding the HUD row, the footer rows and an immediate repeat of
/// `last_char`.
fn spawn_next(s: &mut Session, last_char: Option<char>) -> io::Result<()> {
    let (cols, rows) = term::dims()?;
    let mut rng = rand::rng();
    let mut ch;
    loop {
        ch = rng.random_range(b'a'..=b'z') as char;
        if Some(ch) != last_char {
            break;
        }
    }
    let color = rng.random_range(0..PALETTE.len() as u8);
    let row = if rows > 5 { rng.random_range(2..rows - 2) } else { 2 };
    let col = if cols > 4 { rng.random_range(1..cols - 2) } else { 1 };
    s.spawn_target(ch, row, col, color, Instant::now());
    Ok(())
}

fn render_play(out: &mut impl Write, s: &Session, left: Duration) -> io::Result<()> {
    term::begin_frame(out)?;
    let (cols, rows) = term::dims()?;

    let best = match s.best_reaction {
        Some(d) => format!("{:.2}s   ", d.as_secs_f32()),
        None => "—   ".to_string(),
    };
    let hud = vec![
        ("Điểm: ".to_string(), Color::DarkGrey),
        (format!("{}   ", s.score), Color::Reset),
        ("Chuỗi: ".to_string(), Color::DarkGrey),
        (format!("{}   ", s.streak), Color::Reset),
        ("Còn: ".to_string(), Color::DarkGrey),
        (term::mmss(left), Color::Reset),
        ("   Nhanh nhất: ".to_string(), Color::DarkGrey),
        (best, Color::Reset),
    ];
    print_centered(out, 0, &hud, cols)?;

    if let Some(t) = s.target() {
        let color = PALETTE[t.color as usize % PALETTE.len()];
        queue!(out, MoveTo(t.col, t.row), Print(t.ch.to_string().with(color)))?;
    }

    let footer = vec![
        ("Gõ ký tự đang hiện trên màn hình — gõ sai -20 điểm   ".to_string(), Color::DarkGrey),
        ("Esc".to_string(), Color::Yellow),
        (" dừng ván".to_string(), Color::DarkGrey),
    ];
    print_centered(out, rows.saturating_sub(1), &footer, cols)?;
    out.flush()
}

fn render_summary(out: &mut impl Write, s: &Session, ended_early: bool) -> io::Result<()> {
    term::begin_frame(out)?;
    let (cols, rows) = term::dims()?;

    let title = if ended_early { "ĐÃ DỪNG VÁN" } else { "HẾT GIỜ!" };
    let score_color = match s.score {
        n if n > 0 => Color::Green,
        n if n < 0 => Color::Red,
        _ => Color::Reset,
    };
    let average = match s.average_reaction() {
        Some(d) => format!("{:.2}s", d.as_secs_f32()),
        None => "—".to_string(),
    };
    let best = match s.best_reaction {
        Some(d) => format!("{:.2}s", d.as_secs_f32()),
        None => "—".to_string(),
    };

    let lines: Vec<Vec<(String, Color)>> = vec![
        vec![(title.to_string(), Color::Cyan)],
        vec![("Điểm: ".to_string(), Color::Reset)],
        vec![(format!("{}", s.score), score_color)],
        vec![(
            format!(
                "Đúng: {}   Sai: {}   Chuỗi dài nhất: {}",
                s.hits, s.misses, s.max_streak
            ),
            Color::Reset,
        )],
        vec![(format!("Phản xạ trung bình: {average}   Nhanh nhất: {best}"), Color::Reset)],
        vec![("R".to_string(), Color::Yellow), (" chơi lại   ".to_string(), Color::DarkGrey), ("Esc".to_string(), Color::Yellow), (" về menu".to_string(), Color::DarkGrey)],
    ];

    let start = rows.saturating_sub(lines.len() as u16 + 2) / 2;
    for (i, parts) in lines.iter().enumerate() {
        print_centered(out, start + i as u16, parts, cols)?;
    }
    out.flush()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn render_play_to_string(s: &Session, left: Duration) -> String {
        let mut buf = Vec::new();
        render_play(&mut buf, s, left).unwrap();
        String::from_utf8(buf).unwrap()
    }

    fn render_summary_to_string(s: &Session, early: bool) -> String {
        let mut buf = Vec::new();
        render_summary(&mut buf, s, early).unwrap();
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
    fn play_frame_shows_hud_footer_and_coloured_target() {
        let t0 = Instant::now();
        let mut s = Session::new(t0);
        // PALETTE[3] is Blue → SGR 38;5;12.
        s.spawn_target('z', 5, 10, 3, t0);
        let raw = render_play_to_string(&s, Duration::from_secs(60));
        assert_eq!(raw.matches("\x1b[38;5;12mz").count(), 1, "target in its palette colour");
        let plain = strip_ansi(raw);
        assert!(plain.contains("Điểm:"));
        assert!(plain.contains("Còn:"));
        assert!(plain.contains("Esc"));
    }

    #[test]
    fn target_colour_follows_the_palette_index() {
        let t0 = Instant::now();
        // Palette indices map to bright SGR codes 9..15 in order.
        for (idx, code) in (9..=15).enumerate() {
            let mut s = Session::new(t0);
            s.spawn_target('w', 3, 3, idx as u8, t0);
            let raw = render_play_to_string(&s, Duration::from_secs(60));
            assert!(
                raw.contains(&format!("\x1b[38;5;{code}mw")),
                "palette index {idx} must render as SGR {code}"
            );
        }
    }

    #[test]
    fn consumed_target_disappears_from_the_frame() {
        let t0 = Instant::now();
        let mut s = Session::new(t0);
        s.spawn_target('z', 5, 10, 1, t0); // PALETTE[1] = Green
        s.press('z', t0);
        let raw = render_play_to_string(&s, Duration::from_secs(30));
        assert!(!raw.contains("\x1b[38;5;10mz"), "no target after it was hit");
        assert!(raw.contains("\x1b[38;5;11mEsc"), "footer hint keys stay yellow");
    }

    #[test]
    fn summary_shows_score_and_stats() {
        let t0 = Instant::now();
        let mut s = Session::new(t0);
        s.spawn_target('a', 1, 1, 0, t0);
        s.press('a', t0 + Duration::from_millis(150));
        s.spawn_target('b', 2, 2, 0, t0 + Duration::from_millis(200));
        s.press('x', t0 + Duration::from_millis(300));
        let summary = render_summary_to_string(&s, false);
        assert!(summary.contains("HẾT GIỜ!"));
        assert!(summary.contains("Điểm:"));
        assert!(summary.contains("Đúng: 1   Sai: 1"));
        assert!(summary.contains("Chuỗi dài nhất: 1"));
        assert!(summary.contains("Nhanh nhất: 0.15s"));
    }

    #[test]
    fn early_stop_uses_a_different_title() {
        let s = Session::new(Instant::now());
        let summary = render_summary_to_string(&s, true);
        assert!(summary.contains("ĐÃ DỪNG VÁN"));
        assert!(summary.contains("Đúng: 0"));
    }

    #[test]
    fn only_plain_or_shift_letters_count_as_key_presses() {
        let mut key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(is_plain_key(&key));
        key.modifiers = KeyModifiers::SHIFT;
        assert!(is_plain_key(&key));
        key.modifiers = KeyModifiers::CONTROL;
        assert!(!is_plain_key(&key));
        key.modifiers = KeyModifiers::ALT;
        assert!(!is_plain_key(&key));
    }
}
