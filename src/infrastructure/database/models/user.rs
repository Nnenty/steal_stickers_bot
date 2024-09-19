use crate::domain::entities::user::User as UserEntitie;
use sqlx::{types::time::OffsetDateTime, FromRow};

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct User {
    pub tg_id: i64,
    pub sets_number: i32,
    pub created: OffsetDateTime,
}

impl From<User> for UserEntitie {
    fn from(value: User) -> Self {
        Self {
            tg_id: value.tg_id,
            sets_number: value.sets_number,
            created: value.created,
        }
    }
}
