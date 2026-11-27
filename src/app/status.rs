pub(super) struct Pool {
    pub(super) current: i32,
    pub(super) max: i32,
}

impl Pool {
    pub(super) fn full(max: i32) -> Self {
        Self { current: max, max }
    }
}

pub(super) struct Vitals {
    pub(super) hp: Pool,
    pub(super) mp: Pool,
}
