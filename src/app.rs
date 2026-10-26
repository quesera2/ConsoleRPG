mod ui;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use std::io;

pub struct App {
    player: Player,
    enemies: Vec<Enemy>,
    battle_state: BattleState,
    // 終了フラグ
    should_quit: bool,
}

impl Default for App {
    fn default() -> Self {
        let enemies = (0..3)
            .map(|n| {
                let suffix = char::from_u32(u32::from('Ａ') + n)
                    .expect("'Ａ'に0〜2を足した値は全角英字（Ａ〜Ｃ）なので有効な想定です");
                Enemy {
                    name: format!("スライム{suffix}"),
                    vitals: Vitals {
                        hit_point: Pool::full(15),
                        magic_point: Pool::full(0),
                    },
                }
            })
            .collect();

        Self {
            player: Player {
                name: "ゆうしゃ".to_string(),
                vitals: Vitals {
                    hit_point: Pool::full(30),
                    magic_point: Pool::full(10),
                },
            },
            enemies,
            battle_state: BattleState::CommandSelect,
            should_quit: false,
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            self.draw(terminal)?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        terminal.draw(|frame| frame.render_widget(self, frame.area()))?;
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let Some(key) = crossterm::event::read()?.as_key_press_event() else {
            return Ok(());
        };
        self.handle_key_event(key);
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q' | 'Q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true
            }
            _ => {}
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
    hit_point: Pool,
    magic_point: Pool,
}

struct Player {
    name: String,
    vitals: Vitals,
}

struct Enemy {
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
