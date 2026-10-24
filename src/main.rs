use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(render)?;
        let Some(key) = crossterm::event::read()?.as_key_press_event() else {
            continue;
        };
        match key.code {
            KeyCode::Char('q' | 'Q') => break Ok(()),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
            _ => {}
        }
    }
}

fn render(frame: &mut Frame) {
    let [status_area, enemy_area, message_area] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Fill(1),
        Constraint::Length(6),
    ])
    .areas(frame.area());

    let name = Line::from("ゆうしゃ");
    let hp = Line::from("HP: 30");
    let mp = Line::from("MP: 10");

    let content_width = [&name, &hp, &mp]
        .into_iter()
        .map(Line::width)
        .max()
        .unwrap_or(0);
    let width = u16::try_from(content_width + 2).unwrap_or(u16::MAX);
    frame.render_widget(
        Paragraph::new(vec![hp, mp]).block(Block::default().borders(Borders::ALL).title(name)),
        Rect {
            width,
            ..status_area
        },
    );
}
