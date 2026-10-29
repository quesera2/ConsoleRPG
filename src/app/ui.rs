use super::{
    App, BattleState, Command,
    enemies::{Enemy, EnemyType},
};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, LineGauge, Paragraph, Widget};

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
        let name = Line::from(self.player.name.as_str());
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
        // ゲージ1行 + 隙間1行 + 本体（一番大きい敵サイズをサーチ）
        let height = self
            .enemies
            .iter()
            .map(Enemy::art)
            .map(|art| art.height())
            .max()
            .unwrap_or(0)
            + 2;
        let [row] = Layout::vertical([Constraint::Length(height)])
            .flex(Flex::Center)
            .areas(area);
        let slots = Layout::horizontal(
            self.enemies
                .iter()
                .map(Enemy::art)
                .map(|art| Constraint::Length(art.width())),
        )
        .flex(Flex::Center)
        .spacing(4)
        .split(row);

        for (&slot, enemy) in slots.iter().zip(self.enemies.iter()) {
            let [gauge_area, _, body_area] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas(slot);

            LineGauge::default()
                .ratio(f64::from(enemy.vitals.hp.current) / f64::from(enemy.vitals.hp.max))
                .label("")
                .filled_style(Style::new().fg(Color::Green))
                .unfilled_style(Style::new().fg(Color::DarkGray))
                .render(gauge_area, buf);

            // 背の低い敵も足元が揃うように、アートを下端に寄せる
            let art = enemy.art();
            let [art_area] = Layout::vertical([Constraint::Length(art.height())])
                .flex(Flex::End)
                .areas(body_area);
            Paragraph::new(art.lines).render(art_area, buf);
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
        let is_cursor_visible = match &self.battle_state {
            BattleState::CommandSelect { blink_timer } => blink_timer.is_on(),
        };
        if is_cursor_visible
            && let Some(cursor_row) = command_inner
                .rows()
                .nth(self.cursor_position.command_position)
        {
            Line::from("▶").render(cursor_row, buf)
        }

        let mut groups: Vec<(EnemyType, u16)> = Vec::new();
        for enemy in &self.enemies {
            match groups.iter_mut().find(|(t, _)| *t == enemy.enemy_type) {
                Some((_, count)) => *count += 1,
                None => groups.push((enemy.enemy_type, 1)),
            }
        }

        let enemy_list_area = Rect {
            // 元の確保した高さより大きくならないようにする
            height: enemy_list_area
                .height
                .min(u16::try_from(groups.len()).expect("同時に存在する敵の数は u16 に収まる") + 2),
            ..enemy_list_area
        };
        let enemy_list = Block::bordered();
        let enemy_inner = enemy_list.inner(enemy_list_area);

        enemy_list.render(enemy_list_area, buf);

        let diff = u32::from('０') - u32::from('0');
        for (row, (enemy_type, count)) in enemy_inner.rows().zip(groups) {
            // 同じ行に左寄せと右寄せで重ねて描く
            Line::from(format!("  {name}", name = enemy_type.data().name))
                .left_aligned()
                .render(row, buf);
            let count = count
                .to_string()
                .chars()
                .map(|c| char::from_u32(u32::from(c) + diff).expect("０〜９の範囲に必ず収まる"))
                .collect::<String>();
            Line::from(format!("ー　{count}ひき"))
                .right_aligned()
                .render(row, buf);
        }
    }
}
