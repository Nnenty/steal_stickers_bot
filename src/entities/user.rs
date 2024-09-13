use sqlx::types::time::OffsetDateTime;
use telers::extractors::FromContext;

#[derive(Debug, Clone, PartialEq, Eq, FromContext)]
#[context(key = "db_user")]
pub struct User {
    pub tg_id: i64,
    pub sets_number: i32,
    pub created: OffsetDateTime,
}
