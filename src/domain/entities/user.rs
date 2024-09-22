use sqlx::types::time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub tg_id: i64,
    pub sets_number: i32,
    pub created: OffsetDateTime,
}
