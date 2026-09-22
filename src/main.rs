mod services;
mod tui;
mod utils;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(tui::display::app)?;
    Ok(())
}
