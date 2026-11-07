//! Terminal mini games — a hub of small terminal games.

mod board;
mod games;
mod menu;
mod term;

fn main() -> std::io::Result<()> {
    let _guard = term::TerminalGuard::new()?;
    menu::run()
}
