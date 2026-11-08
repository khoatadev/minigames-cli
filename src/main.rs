//! Terminal mini games — a hub of small terminal games.

use std::io;

mod board;
mod games;
mod menu;
mod reaction;
mod term;

fn main() -> std::io::Result<()> {
    use crossterm::tty::IsTty;
    if !io::stdout().is_tty() {
        eprintln!("game-cli cần một terminal tương tác — hãy chạy nó trong terminal.");
        std::process::exit(1);
    }
    let _guard = term::TerminalGuard::new()?;
    menu::run()
}
