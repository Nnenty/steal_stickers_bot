use crate::entities::set::Set as SetEntitie;
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Set {
    pub tg_id: i64,
    pub short_name: String,
    pub title: String,
}

impl<'a> From<Set> for SetEntitie {
    fn from(value: Set) -> Self {
        Self {
            tg_id: value.tg_id,
            short_name: value.short_name,
            title: value.title,
        }
    }
}
