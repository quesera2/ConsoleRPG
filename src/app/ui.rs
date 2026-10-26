use super::{App, Command};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Margin, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, LineGauge, Paragraph, Widget};

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

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [status_area, enemy_area, message_area] = Layout::vertical([
            Constraint::Length(4),
            Constraint::Fill(1),
            Constraint::Length(6),
        ])
        .areas(area);

        self.render_status(status_area, buf);
        self.render_enemies(enemy_area, buf);
        self.render_command(message_area, buf);
    }
}

impl App {
    fn render_status(&self, area: Rect, buf: &mut Buffer) {
        let name = Line::from(self.player.name.to_string());
        let hp = Line::from(format!("HP: {hp}", hp = self.player.vitals.hp.current));
        let mp = Line::from(format!("MP: {mp}", mp = self.player.vitals.mp.current));

        let content_width = [&name, &hp, &mp]
            .into_iter()
            .map(Line::width)
            .max()
            .unwrap_or(0);
        let width = u16::try_from(content_width + 2).unwrap_or(u16::MAX);
        Paragraph::new(vec![hp, mp])
            .block(Block::bordered().title(name))
            .render(Rect { width, ..area }, buf);
    }

    fn render_enemies(&self, area: Rect, buf: &mut Buffer) {
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

            LineGauge::default()
                .ratio(1.0)
                .label("")
                .filled_style(Style::new().fg(Color::Green))
                .unfilled_style(Style::new().fg(Color::DarkGray))
                .render(gauge_area, buf);

            Paragraph::new(SLIME_ART.as_slice()).render(body_area, buf);
        }
    }

    fn render_command(&self, area: Rect, buf: &mut Buffer) {
        let commands: Vec<Line> = Command::ALL
            .iter()
            .map(|c| format!("  {display_name}", display_name = c.display_name()))
            .map(Line::from)
            .collect();

        let content_width = commands.iter().map(Line::width).max().unwrap_or(0);
        let command_width = u16::try_from(content_width + 2).unwrap_or(u16::MAX);

        let [command_area, _, enemy_list_area] = Layout::horizontal([
            Constraint::Length(command_width),
            Constraint::Length(2),
            Constraint::Fill(1),
        ])
        .areas(area);

        let command_list = Block::bordered().title("コマンド");
        let command_inner = command_list.inner(command_area);

        command_list.render(command_area, buf);
        Paragraph::new(commands).render(command_inner, buf);
        let position = Margin {
            horizontal: 0,
            vertical: 0,
        };
        Line::from("▶︎").render(command_inner.inner(position), buf);

        let enemy_list_area = Rect {
            height: 3,
            ..enemy_list_area
        };
        let enemy_list = Block::bordered();
        let enemy_inner = enemy_list.inner(enemy_list_area);

        enemy_list.render(enemy_list_area, buf);

        // 同じ行に左寄せと右寄せで重ねて描く
        Line::from("　スライム")
            .left_aligned()
            .render(enemy_inner, buf);
        Line::from("ー　３ひき")
            .right_aligned()
            .render(enemy_inner, buf);
    }
}
