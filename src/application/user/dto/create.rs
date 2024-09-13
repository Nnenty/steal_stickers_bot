use sqlx::prelude::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Create {
    tg_id: i64,
    sets_number: i32,
}

impl Create {
    pub const fn new(tg_id: i64, sets_number: i32) -> Self {
        Self { tg_id, sets_number }
    }
    pub const fn tg_id(&self) -> i64 {
        self.tg_id
    }
    pub const fn sets_number(&self) -> i32 {
        self.sets_number
    }
}
