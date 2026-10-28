mod enemies;
mod ui;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use enemies::{Enemy, EnemyType};
use ratatui::DefaultTerminal;
use std::io;

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
            battle_state: BattleState::CommandSelect,
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

    fn handle_events() -> io::Result<Option<Input>> {
        let event = crossterm::event::read()?;
        Ok(event.as_key_press_event().and_then(Input::from_key_event))
    }

    fn handle_input(&mut self, input: Option<Input>) {
        let Some(input) = input else { return };
        match input {
            Input::System(SystemInput::Quit) => self.should_quit = true,
            Input::Game(game_input) => match self.battle_state {
                BattleState::CommandSelect => self.handle_command_select_input(game_input),
            },
        }
    }

    fn handle_command_select_input(&mut self, input: GameInput) {
        match input {
            GameInput::Up => {
                self.cursor_position.command_position =
                    self.cursor_position.command_position.saturating_sub(1)
            }
            GameInput::Down => {
                self.cursor_position.command_position =
                    (self.cursor_position.command_position + 1).min(Command::ALL.len() - 1)
            }

            GameInput::Left | GameInput::Right | GameInput::Confirm | GameInput::Cancel => {}
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
    CommandSelect,
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
        assert_eq!(Input::from_key_event(key), Some(Input::System(SystemInput::Quit)));
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
}
