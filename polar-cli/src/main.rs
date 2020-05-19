pub mod cli;
mod repl;
mod tui;

fn main() -> anyhow::Result<()> {
    if cfg!(feature = "repl") {
        repl::main()?;
    }
    if cfg!(feature = "tui_") {
        tui::main()?;
    }
    Ok(())
}
