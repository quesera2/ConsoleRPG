use super::{Pool, Vitals};
use ratatui::text::Line;

// 敵の見た目。各行の表示幅をそろえておく
#[rustfmt::skip]
const SLIME_ART: &[&str] = &[
    r"    _.-----._    ",
    r"  .'         `.  ",
    r" /   ●     ●   \ ",
    r"|       ▽       |",
    r"|               |",
    r" `-.._______..-' ",
];

// 種族ごとに共通のデータ
pub(super) struct EnemyData {
    pub(super) name: &'static str,
    pub(super) max_hp: i32,
    pub(super) max_mp: i32,
    pub(super) art: EnemyArt,
}

pub(super) struct EnemyArt {
    pub(super) lines: &'static [&'static str],
}

impl EnemyArt {
    pub(super) fn width(&self) -> u16 {
        self.lines
            .iter()
            .copied()
            .map(|line| Line::from(line).width())
            .max()
            .and_then(|max| u16::try_from(max).ok())
            .expect("アートは１行以上あり、幅はu16に収まる")
    }

    pub(super) fn height(&self) -> u16 {
        u16::try_from(self.lines.len()).expect("アートの高さはu16に収まる")
    }
}

const SLIME: EnemyData = EnemyData {
    name: "スライム",
    max_hp: 15,
    max_mp: 0,
    art: EnemyArt { lines: SLIME_ART },
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum EnemyType {
    Slime,
}

impl EnemyType {
    pub(super) fn data(self) -> &'static EnemyData {
        match self {
            EnemyType::Slime => &SLIME,
        }
    }
}

// 個体ごとのデータ
pub(super) struct Enemy {
    pub(super) enemy_type: EnemyType,
    // 同じ種族が複数いるときの識別番号（0ならＡ）。1体だけならNone
    pub(super) suffix: Option<u8>,
    pub(super) vitals: Vitals,
}

impl Enemy {
    pub(super) fn new(enemy_type: EnemyType, suffix: Option<u8>) -> Self {
        let data = enemy_type.data();
        Self {
            enemy_type,
            suffix,
            vitals: Vitals {
                hp: Pool::full(data.max_hp),
                mp: Pool::full(data.max_mp),
            },
        }
    }

    // 「スライムＡ」のような表示用の名前
    pub(super) fn display_name(&self) -> String {
        let name = self.enemy_type.data().name;
        match self.suffix {
            Some(n) => {
                let suffix = char::from_u32(u32::from('Ａ') + u32::from(n))
                    .expect("同じ種族は26体までの前提なので、'Ａ'に足しても全角英字の範囲に収まる");
                format!("{name}{suffix}")
            }
            None => name.to_string(),
        }
    }

    pub(super) fn art(&self) -> &'static EnemyArt {
        &self.enemy_type.data().art
    }
}
