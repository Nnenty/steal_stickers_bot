use telers::extractors::FromContext;

#[derive(Debug, Clone, PartialEq, Eq, FromContext)]
#[context(key = "db_set")]
pub struct Set {
    pub tg_id: i64,
    pub short_name: String,
    pub title: String,
}
