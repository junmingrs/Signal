mod tui;
mod services;
mod utils;
mod database;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(tui::display::app)?;
    Ok(())
}
