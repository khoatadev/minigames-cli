mod board;
mod games;
mod term;

use games::minesweeper;

fn main() -> std::io::Result<()> {
    let _guard = term::TerminalGuard::new()?;
    minesweeper::run(&minesweeper::DIFFICULTIES[0])
}
