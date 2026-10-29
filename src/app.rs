mod enemies;
mod ui;

use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use enemies::{Enemy, EnemyGroup, EnemyType};
use ratatui::DefaultTerminal;
use std::io;
use std::time::{Duration, Instant};

const TICK_RATE: u64 = 30;
const TICK_INTERVAL: Duration = Duration::from_millis(1000 / TICK_RATE);

const BLINK_PERIOD: u128 = 666;

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
            self.draw(terminal)?;
            let input = Self::handle_events()?;
            self.handle_input(input);
        }
        Ok(())
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
        debug_assert!(groups.len() >= 1);
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
                let Some(group) = groups.get(self.cursor_position.enemy_group_position) else {
                    return;
                };
                // 敵選択後どうするかはまだ決めていない
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

enum BattleState {
    // コマンド選択
    CommandSelect { blink_timer: BlinkTimer },
    // こうげきの敵選択
    AttackEnemySelect { blink_timer: BlinkTimer },
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

#[derive(Clone, Copy, PartialEq, Debug)]
enum Input {
    System(SystemInput),
    Game(GameInput),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum GameInput {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum SystemInput {
    Quit,
}

impl Input {
    fn from_key_event(key: KeyEvent) -> Option<Input> {
        match key.code {
            KeyCode::Char('q' | 'Q') => Some(Input::System(SystemInput::Quit)),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Input::System(SystemInput::Quit))
            }
            KeyCode::Up => Some(Input::Game(GameInput::Up)),
            KeyCode::Down => Some(Input::Game(GameInput::Down)),
            KeyCode::Left => Some(Input::Game(GameInput::Left)),
            KeyCode::Right => Some(Input::Game(GameInput::Right)),
            KeyCode::Enter | KeyCode::Char('z' | 'Z') => Some(Input::Game(GameInput::Confirm)),
            KeyCode::Esc | KeyCode::Char('x' | 'X') => Some(Input::Game(GameInput::Cancel)),
            _ => None,
        }
    }
}

#[derive(Default)]
struct CursorPosition {
    command_position: usize,
    enemy_group_position: usize,
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
    fn q_keys_map_to_quit() {
        for key in [KeyCode::Char('q'), KeyCode::Char('Q')] {
            let input = Input::from_key_event(KeyEvent::from(key));
            assert_eq!(input, Some(Input::System(SystemInput::Quit)), "{key:?}");
        }
    }

    #[test]
    fn ctrl_c_maps_to_quit() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(
            Input::from_key_event(key),
            Some(Input::System(SystemInput::Quit))
        );
    }

    #[test]
    fn c_without_ctrl_maps_to_none() {
        let key = KeyEvent::from(KeyCode::Char('c'));
        assert_eq!(Input::from_key_event(key), None);
    }

    #[test]
    fn arrow_keys_map_to_direction() {
        for (key, expect) in [
            (KeyCode::Up, GameInput::Up),
            (KeyCode::Down, GameInput::Down),
            (KeyCode::Left, GameInput::Left),
            (KeyCode::Right, GameInput::Right),
        ] {
            let input = Input::from_key_event(KeyEvent::from(key));
            assert_eq!(input, Some(Input::Game(expect)), "{key:?}");
        }
    }

    #[test]
    fn confirm_keys_map_to_confirm() {
        for key in [KeyCode::Enter, KeyCode::Char('z'), KeyCode::Char('Z')] {
            let input = Input::from_key_event(KeyEvent::from(key));
            assert_eq!(input, Some(Input::Game(GameInput::Confirm)), "{key:?}");
        }
    }

    #[test]
    fn cancel_keys_map_to_cancel() {
        for key in [KeyCode::Esc, KeyCode::Char('x'), KeyCode::Char('X')] {
            let input = Input::from_key_event(KeyEvent::from(key));
            assert_eq!(input, Some(Input::Game(GameInput::Cancel)), "{key:?}");
        }
    }

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
}
