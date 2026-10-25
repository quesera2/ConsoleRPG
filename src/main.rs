mod app;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, LineGauge, Paragraph};
use ratatui::{DefaultTerminal, Frame};

// 敵の見た目。各行の表示幅をそろえておく
#[rustfmt::skip]
const SLIME_ART: [&str; 6] = [
    r"    _.-----._    ",
    r"  .'         `.  ",
    r" /   ●     ●   \ ",
    r"|       ▽       |",
    r"|               |",
    r" `-.._______..-' ",
];
const ENEMY_WIDTH: u16 = 17;

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

    render_status(frame, status_area);
    render_enemies(frame, enemy_area);
    render_command(frame, message_area);
}

fn render_status(frame: &mut Frame, area: Rect) {
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
        Paragraph::new(vec![hp, mp]).block(Block::bordered().title(name)),
        Rect { width, ..area },
    );
}

fn render_enemies(frame: &mut Frame, area: Rect) {
    // ゲージ1行 + 隙間1行 + 本体
    let height = 2 + SLIME_ART.len() as u16;
    let [row] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    let slots: [Rect; 3] = Layout::horizontal([Constraint::Length(ENEMY_WIDTH); 3])
        .flex(Flex::Center)
        .spacing(4)
        .areas(row);

    for slot in slots {
        let [gauge_area, _, body_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(slot);

        frame.render_widget(
            LineGauge::default()
                .ratio(1.0)
                .label("")
                .filled_style(Style::new().fg(Color::Green))
                .unfilled_style(Style::new().fg(Color::DarkGray)),
            gauge_area,
        );
        frame.render_widget(Paragraph::new(SLIME_ART.as_slice()), body_area);
    }
}

fn render_command(frame: &mut Frame, area: Rect) {
    let commands = vec![
        Line::from("> たたかう"),
        Line::from("  じゅもん"),
        Line::from("  にげる"),
    ];
    let content_width = commands.iter().map(Line::width).max().unwrap_or(0);
    let command_width = u16::try_from(content_width + 2).unwrap_or(u16::MAX);

    let [command_area, _, enemy_list_area] = Layout::horizontal([
        Constraint::Length(command_width),
        Constraint::Length(2),
        Constraint::Fill(1),
    ])
    .areas(area);

    frame.render_widget(
        Paragraph::new(commands).block(Block::bordered().title("コマンド")),
        command_area,
    );

    let enemy_list_area = Rect {
        height: 3,
        ..enemy_list_area
    };
    let enemy_list = Block::bordered();
    let inner = enemy_list.inner(enemy_list_area);
    frame.render_widget(enemy_list, enemy_list_area);

    // 同じ行に左寄せと右寄せで重ねて描く
    frame.render_widget(Line::from("　スライム").left_aligned(), inner);
    frame.render_widget(Line::from("ー　３ひき").right_aligned(), inner);
}
