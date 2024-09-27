use sqlx::types::time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub tg_id: i64,
    pub created: OffsetDateTime,
}
