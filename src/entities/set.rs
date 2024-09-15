use telers::extractors::FromContext;

#[derive(Debug, Clone, PartialEq, Eq, FromContext)]
#[context(key = "db_set")]
pub struct Set<'a> {
    pub tg_id: i64,
    pub short_name: &'a str,
    pub title: &'a str,
}
