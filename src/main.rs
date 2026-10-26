mod app;

use app::App;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut app = App::default();
    ratatui::run(|t| app.run(t))?;
    Ok(())
}
