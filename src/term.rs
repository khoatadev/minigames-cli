//! Shared terminal helpers: setup/teardown and input polling.

use std::io::{self, Write, stdout};
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size};

/// How long the input loop waits before re-checking the clock.
pub const POLL_MS: Duration = Duration::from_millis(100);

/// Restores the terminal (raw mode off, cursor back, main screen) on drop —
/// even if a game panics.
pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), crossterm::cursor::Show, LeaveAlternateScreen);
    }
}

/// Clears the frame and moves the cursor to the top-left.
pub fn begin_frame(out: &mut impl Write) -> io::Result<()> {
    execute!(out, Clear(ClearType::All), crossterm::cursor::MoveTo(0, 0))
}

/// Top-left corner `(x, y)` that centers a `content_w` x `content_h` block
/// in the terminal. Clamps to 0 when the terminal is too small.
pub fn center_offset(content_w: u16, content_h: u16) -> io::Result<(u16, u16)> {
    let (cols, rows) = size()?;
    Ok((
        cols.saturating_sub(content_w) / 2,
        rows.saturating_sub(content_h) / 2,
    ))
}

/// Terminal size as `(cols, rows)`.
pub fn dims() -> io::Result<(u16, u16)> {
    size()
}

/// Polls for one event, or `None` if nothing arrived within `timeout`.
pub fn poll_event(timeout: Duration) -> io::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}


