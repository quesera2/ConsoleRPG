mod enemies;
mod input;
mod ui;

use crossterm::event::{self};
use enemies::{Enemy, EnemyGroup, EnemyType};
use input::{GameInput, Input, SystemInput};
use ratatui::DefaultTerminal;
use std::collections::VecDeque;
use std::io;
use std::time::{Duration, Instant};

const TICK_RATE: u64 = 30;
const TICK_INTERVAL: Duration = Duration::from_millis(1000 / TICK_RATE);

const BLINK_PERIOD: u128 = 666;

const INITIAL_MESSAGE_INTERVAL: Duration = Duration::from_millis(200);
const MESSAGE_INTERVAL: Duration = Duration::from_millis(400);

pub struct App {
    player: Player,
    enemies: Vec<Enemy>,
    battle_state: BattleState,
    // 終了フラグ
    should_quit: bool,
    cursor_position: CursorPosition,
}

impl Default for App {
    fn default() -> Self {
        // 同じ種族が3体なので、全員に識別番号を振る
        let enemies = (0..3)
            .map(|n| Enemy::new(EnemyType::Slime, Some(n)))
            .collect();

        Self {
            player: Player {
                name: "ゆうしゃ".to_string(),
                vitals: Vitals {
                    hp: Pool::full(30),
                    mp: Pool::full(10),
                },
            },
            enemies,
            battle_state: BattleState::CommandSelect {
                blink_timer: BlinkTimer::start(),
            },
            should_quit: false,
            cursor_position: CursorPosition::default(),
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            self.update(Instant::now());
            self.draw(terminal)?;
            let input = Self::handle_events()?;
            self.handle_input(input);
        }
        Ok(())
    }

    fn update(&mut self, now: Instant) {
        match &mut self.battle_state {
            BattleState::CommandSelect { .. } | BattleState::AttackEnemySelect { .. } => {}
            BattleState::DisplayMessage {
                pending_events,
                messages,
                last_shown_at,
                show_next,
                blink_timer,
            } => {
                let wait_time = if messages.is_empty() {
                    INITIAL_MESSAGE_INTERVAL
                } else {
                    MESSAGE_INTERVAL
                };
                if *show_next || now.duration_since(*last_shown_at) <= wait_time {
                    return;
                }
                match pending_events.pop_front() {
                    Some(BattleEvent::Message(text)) => {
                        messages.push(text);
                        *last_shown_at = now;
                    }
                    Some(BattleEvent::PageBreak) => {
                        messages.clear();
                        *last_shown_at = now;
                    }
                    None => {
                        *show_next = true;
                        *blink_timer = BlinkTimer::start();
                    }
                }
            }
        }
    }

    fn draw(&self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        terminal.draw(|frame| frame.render_widget(self, frame.area()))?;
        Ok(())
    }

    fn enemy_groups(&self) -> Vec<EnemyGroup> {
        let mut groups: Vec<EnemyGroup> = Vec::new();
        //TODO: 敵の生存フラグを追加したら生存している場合のみ加算するように修正
        for enemy in &self.enemies {
            match groups
                .iter_mut()
                .find(|group| group.enemy_type == enemy.enemy_type)
            {
                Some(group) => group.count += 1,
                None => groups.push(EnemyGroup {
                    enemy_type: enemy.enemy_type,
                    count: 1,
                }),
            }
        }
        groups
    }

    fn handle_events() -> io::Result<Option<Input>> {
        if event::poll(TICK_INTERVAL)? {
            let event = event::read()?;
            Ok(event.as_key_press_event().and_then(Input::from_key_event))
        } else {
            Ok(None)
        }
    }

    fn handle_input(&mut self, input: Option<Input>) {
        let Some(input) = input else { return };
        match input {
            Input::System(SystemInput::Quit) => self.should_quit = true,
            Input::Game(game_input) => match self.battle_state {
                BattleState::CommandSelect { .. } => self.handle_command_select_input(game_input),
                BattleState::AttackEnemySelect { .. } => self.handle_enemy_select_input(game_input),
                BattleState::DisplayMessage { show_next, .. } => {
                    self.handle_command_display_message(show_next, game_input)
                }
            },
        }
    }

    fn handle_command_select_input(&mut self, input: GameInput) {
        let previous_cursor_position = self.cursor_position.command_position;
        match input {
            GameInput::Up => {
                self.cursor_position.command_position =
                    self.cursor_position.command_position.saturating_sub(1)
            }
            GameInput::Down => {
                self.cursor_position.command_position =
                    (self.cursor_position.command_position + 1).min(Command::ALL.len() - 1)
            }
            GameInput::Confirm => {
                let Some(current_command) = Command::ALL.get(self.cursor_position.command_position)
                else {
                    return;
                };
                match current_command {
                    Command::Attack => {
                        self.battle_state = BattleState::AttackEnemySelect {
                            blink_timer: BlinkTimer::start(),
                        }
                    }
                    Command::Magic | Command::Flee => {}
                }
            }
            GameInput::Left | GameInput::Right | GameInput::Cancel => {}
        }
        if previous_cursor_position != self.cursor_position.command_position {
            self.battle_state = BattleState::CommandSelect {
                blink_timer: BlinkTimer::start(),
            };
        }
    }

    fn handle_enemy_select_input(&mut self, input: GameInput) {
        let previous_cursor_position = self.cursor_position.enemy_group_position;
        let groups = self.enemy_groups();
        debug_assert!(!groups.is_empty(), "敵選択時、必ず１グループ以上存在する");
        match input {
            GameInput::Up => {
                self.cursor_position.enemy_group_position =
                    self.cursor_position.enemy_group_position.saturating_sub(1)
            }
            GameInput::Down => {
                self.cursor_position.enemy_group_position =
                    (self.cursor_position.enemy_group_position + 1).min(groups.len() - 1)
            }
            GameInput::Confirm => {
                let Some(_) = groups.get(self.cursor_position.enemy_group_position) else {
                    return;
                };
                // TODO: ダミーメッセージを入れており、後から戦闘処理に差し替える
                self.battle_state = BattleState::DisplayMessage {
                    pending_events: VecDeque::from([
                        BattleEvent::Message("ゆうしゃのこうげき！".to_string()),
                        BattleEvent::Message("スライムＡに5のダメージを与えた！".to_string()),
                    ]),
                    messages: Vec::new(),
                    last_shown_at: Instant::now(),
                    show_next: false,
                    blink_timer: BlinkTimer::start(),
                };
                return;
            }
            GameInput::Cancel => {
                self.battle_state = BattleState::CommandSelect {
                    blink_timer: BlinkTimer::start(),
                };
            }
            GameInput::Left | GameInput::Right => {}
        }
        if previous_cursor_position != self.cursor_position.enemy_group_position {
            self.battle_state = BattleState::AttackEnemySelect {
                blink_timer: BlinkTimer::start(),
            };
        }
    }

    fn handle_command_display_message(&mut self, show_next: bool, input: GameInput) {
        match input {
            GameInput::Confirm => {
                if show_next {
                    self.cursor_position.reset();
                    self.battle_state = BattleState::CommandSelect {
                        blink_timer: BlinkTimer::start(),
                    };
                }
            }
            GameInput::Up
            | GameInput::Down
            | GameInput::Left
            | GameInput::Right
            | GameInput::Cancel => {}
        }
    }
}

struct Pool {
    current: i32,
    max: i32,
}

impl Pool {
    fn full(max: i32) -> Self {
        Self { current: max, max }
    }
}

struct Vitals {
    hp: Pool,
    mp: Pool,
}

struct Player {
    name: String,
    vitals: Vitals,
}

enum BattleEvent {
    /// 戦闘メッセージ
    Message(String),
    /// メッセージをクリア
    PageBreak,
}

enum BattleState {
    /// コマンド選択
    CommandSelect { blink_timer: BlinkTimer },
    /// こうげきの敵選択
    AttackEnemySelect { blink_timer: BlinkTimer },
    /// 戦闘メッセージ表示
    DisplayMessage {
        /// 未処理イベント
        pending_events: VecDeque<BattleEvent>,
        /// 表示中メッセージ
        messages: Vec<String>,
        /// 最後に表示を更新した時間
        last_shown_at: Instant,
        /// 次へカーソルを表示するかどうか
        show_next: bool,
        /// 次へカーソルの点滅用
        blink_timer: BlinkTimer,
    },
}

enum Command {
    Attack,
    Magic,
    Flee,
}

impl Command {
    const ALL: [Command; 3] = [Command::Attack, Command::Magic, Command::Flee];

    fn display_name(&self) -> &'static str {
        match self {
            Command::Attack => "たたかう",
            Command::Magic => "じゅもん",
            Command::Flee => "にげる",
        }
    }
}

#[derive(Default)]
struct CursorPosition {
    command_position: usize,
    enemy_group_position: usize,
}

impl CursorPosition {
    fn reset(&mut self) {
        self.command_position = 0;
        self.enemy_group_position = 0;
    }
}

struct BlinkTimer {
    started_at: Instant,
}

impl BlinkTimer {
    fn start() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }

    fn is_on(&self) -> bool {
        self.started_at.elapsed().as_millis() % BLINK_PERIOD < BLINK_PERIOD / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quit_sets_should_quit() {
        let mut app = App::default();
        app.handle_input(Some(Input::System(SystemInput::Quit)));
        assert!(app.should_quit);
    }

    #[test]
    fn down_moves_command_cursor() {
        let mut app = App::default();
        app.handle_input(Some(Input::Game(GameInput::Down)));
        assert_eq!(app.cursor_position.command_position, 1);
    }

    #[test]
    fn up_moves_command_cursor() {
        let mut app = App::default();
        app.cursor_position.command_position = 1;
        app.handle_input(Some(Input::Game(GameInput::Up)));
        assert_eq!(app.cursor_position.command_position, 0);
    }

    #[test]
    fn up_at_top_stays_at_top() {
        let mut app = App::default();
        app.handle_input(Some(Input::Game(GameInput::Up)));
        assert_eq!(app.cursor_position.command_position, 0);
    }

    #[test]
    fn down_at_bottom_stays_at_bottom() {
        let mut app = App::default();
        let last = Command::ALL.len() - 1;
        app.cursor_position.command_position = last;
        app.handle_input(Some(Input::Game(GameInput::Down)));
        assert_eq!(app.cursor_position.command_position, last);
    }

    #[test]
    fn confirm_attack_moves_to_enemy_select() {
        let mut app = App::default();
        // 初期位置は先頭の「たたかう」
        app.handle_input(Some(Input::Game(GameInput::Confirm)));
        assert!(matches!(
            app.battle_state,
            BattleState::AttackEnemySelect { .. }
        ));
    }

    #[test]
    fn cancel_in_enemy_select_returns_to_command_select() {
        let mut app = App {
            battle_state: BattleState::AttackEnemySelect {
                blink_timer: BlinkTimer::start(),
            },
            ..App::default()
        };
        app.handle_input(Some(Input::Game(GameInput::Cancel)));
        assert!(matches!(
            app.battle_state,
            BattleState::CommandSelect { .. }
        ));
    }

    #[test]
    fn confirm_in_enemy_select_moves_to_display_message() {
        let mut app = App {
            battle_state: BattleState::AttackEnemySelect {
                blink_timer: BlinkTimer::start(),
            },
            ..App::default()
        };
        app.handle_input(Some(Input::Game(GameInput::Confirm)));
        assert!(matches!(
            app.battle_state,
            BattleState::DisplayMessage { .. }
        ));
    }

    #[test]
    fn confirm_in_display_message_returns_to_command_select() {
        let mut app = App {
            battle_state: BattleState::DisplayMessage {
                pending_events: VecDeque::default(),
                messages: vec!["メッセージ".to_string()],
                blink_timer: BlinkTimer::start(),
                show_next: true,
                last_shown_at: Instant::now(),
            },
            ..App::default()
        };
        app.handle_input(Some(Input::Game(GameInput::Confirm)));
        assert!(matches!(
            app.battle_state,
            BattleState::CommandSelect { .. }
        ));
    }

    #[test]
    fn confirm_in_display_message_resets_cursor_position() {
        let mut app = App {
            battle_state: BattleState::DisplayMessage {
                pending_events: VecDeque::default(),
                messages: vec!["メッセージ".to_string()],
                blink_timer: BlinkTimer::start(),
                show_next: true,
                last_shown_at: Instant::now(),
            },
            cursor_position: CursorPosition {
                command_position: 1,
                enemy_group_position: 1,
            },
            ..App::default()
        };
        app.handle_input(Some(Input::Game(GameInput::Confirm)));
        assert_eq!(app.cursor_position.command_position, 0);
        assert_eq!(app.cursor_position.enemy_group_position, 0);
    }

    #[test]
    fn confirm_before_all_messages_shown_is_ignored() {
        let mut app = App {
            battle_state: BattleState::DisplayMessage {
                pending_events: VecDeque::from([BattleEvent::Message("つぎ".to_string())]),
                messages: vec!["メッセージ".to_string()],
                blink_timer: BlinkTimer::start(),
                show_next: false,
                last_shown_at: Instant::now(),
            },
            ..App::default()
        };
        app.handle_input(Some(Input::Game(GameInput::Confirm)));
        assert!(matches!(
            app.battle_state,
            BattleState::DisplayMessage { .. }
        ));
    }

    /// メッセージ表示中の App を作るヘルパー。基準時刻 `base` を返り値でも返す
    fn display_message_app(pending: Vec<BattleEvent>, messages: Vec<&str>) -> (App, Instant) {
        let base = Instant::now();
        let app = App {
            battle_state: BattleState::DisplayMessage {
                pending_events: VecDeque::from(pending),
                messages: messages.into_iter().map(String::from).collect(),
                blink_timer: BlinkTimer::start(),
                show_next: false,
                last_shown_at: base,
            },
            ..App::default()
        };
        (app, base)
    }

    /// 表示済みの行と ▼ を表示するかを取り出す
    fn shown(app: &App) -> (Vec<String>, bool) {
        match &app.battle_state {
            BattleState::DisplayMessage {
                messages,
                show_next,
                ..
            } => (messages.clone(), *show_next),
            _ => panic!("DisplayMessage ではない"),
        }
    }

    const ONE_MS: Duration = Duration::from_millis(1);

    #[test]
    fn update_waits_initial_interval_before_first_line() {
        let (mut app, base) =
            display_message_app(vec![BattleEvent::Message("1行目".to_string())], vec![]);

        app.update(base + INITIAL_MESSAGE_INTERVAL);
        assert_eq!(shown(&app).0, Vec::<String>::new());

        app.update(base + INITIAL_MESSAGE_INTERVAL + ONE_MS);
        assert_eq!(shown(&app).0, vec!["1行目"]);
    }

    #[test]
    fn update_waits_message_interval_after_first_line() {
        let (mut app, base) = display_message_app(
            vec![BattleEvent::Message("2行目".to_string())],
            vec!["1行目"],
        );

        // 1行目用の短い待ち時間では、まだ出ない
        app.update(base + INITIAL_MESSAGE_INTERVAL + ONE_MS);
        assert_eq!(shown(&app).0, vec!["1行目"]);

        app.update(base + MESSAGE_INTERVAL + ONE_MS);
        assert_eq!(shown(&app).0, vec!["1行目", "2行目"]);
    }

    #[test]
    fn update_shows_next_marker_after_all_lines() {
        let (mut app, base) = display_message_app(vec![], vec!["1行目"]);

        app.update(base + MESSAGE_INTERVAL);
        assert!(!shown(&app).1);

        app.update(base + MESSAGE_INTERVAL + ONE_MS);
        assert!(shown(&app).1);
    }

    #[test]
    fn update_clears_lines_on_page_break() {
        let (mut app, base) = display_message_app(
            vec![
                BattleEvent::PageBreak,
                BattleEvent::Message("次のページ".to_string()),
            ],
            vec!["1行目", "2行目"],
        );

        let cleared_at = base + MESSAGE_INTERVAL + ONE_MS;
        app.update(cleared_at);
        assert_eq!(shown(&app).0, Vec::<String>::new());

        // 改ページ後の1行目は、改ページの時刻から1行目用の待ち時間で出る
        app.update(cleared_at + INITIAL_MESSAGE_INTERVAL + ONE_MS);
        assert_eq!(shown(&app).0, vec!["次のページ"]);
    }
}
