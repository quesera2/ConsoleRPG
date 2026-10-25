pub struct App {
    player: Player,
    enemies: Vec<Enemy>,
    battle_state: BattleState,
    // 終了フラグ
    should_quit: bool,
}

struct Status {
    hit_point: i32,
    max_hit_point: i32,
    magic_point: i32,
    max_magic_point: i32,
}

struct Player {
    name: String,
    status: Status,
}

struct Enemy {
    name: String,
    status: Status,
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
