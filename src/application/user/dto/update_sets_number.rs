use sqlx::prelude::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct UpdateSetsNumber {
    tg_id: i64,
    sets_number: i32,
}

impl UpdateSetsNumber {
    pub const fn new(sets_number: i32, tg_id: i64) -> Self {
        Self { sets_number, tg_id }
    }
    pub const fn sets_number(&self) -> i32 {
        self.sets_number
    }
    pub const fn tg_id(&self) -> i64 {
        self.tg_id
    }
}
