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
                        hit_point: Pool::max(15),
                        magic_point: Pool::max(0),
                    },
                }
            })
            .collect();

        Self {
            player: Player {
                name: "ゆうしゃ".to_string(),
                vitals: Vitals {
                    hit_point: Pool::max(30),
                    magic_point: Pool::max(10),
                },
            },
            enemies,
            battle_state: BattleState::CommandSelect,
            should_quit: false,
        }
    }
}

struct Pool {
    current: i32,
    max: i32,
}

impl Pool {
    fn max(value: i32) -> Self {
        Self {
            current: value,
            max: value,
        }
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
