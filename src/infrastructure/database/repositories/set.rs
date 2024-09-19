use async_trait::async_trait;
use sea_query::{Alias, Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::PgConnection;
use tracing::debug;

use crate::{
    application::{
        common::exceptions::RepoKind,
        set::{
            dto::{
                create::Create, delete_by_short_name::DeleteByShortName,
                get_by_short_name::GetByShortName, get_by_tg_id::GetByTgID,
            },
            exceptions::{SetShortNameAlreadyExist, SetShortNameNotExist, SetTgIdNotExist},
            traits::SetRepo,
        },
    },
    domain::entities::set::Set,
    infrastructure::database::models::set::Set as SetModel,
};

pub struct SetRepoImpl<Conn> {
    conn: Conn,
}

impl<Conn> SetRepoImpl<Conn> {
    pub fn new(conn: Conn) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl<'b> SetRepo for SetRepoImpl<&'b mut PgConnection> {
    async fn create<'a>(
        &'a mut self,
        set: Create<'a>,
    ) -> Result<(), RepoKind<SetShortNameAlreadyExist<'a>>> {
        let (sql_query, values) = Query::insert()
            .into_table(Alias::new("sets"))
            .columns([
                Alias::new("tg_id"),
                Alias::new("short_name"),
                Alias::new("title"),
            ])
            .values_panic([
                set.tg_id().into(),
                set.short_name().into(),
                set.title().into(),
            ])
            .build_sqlx(PostgresQueryBuilder);

        debug!("SQL query: {sql_query};\nValues for query: {values:?}");

        sqlx::query_with(&sql_query, values)
            .execute(&mut *self.conn)
            .await
            .map(|_| ())
            .map_err(|err| {
                if let Some(err) = err.as_database_error() {
                    if let Some(code) = err.code() {
                        if code == "23505" {
                            return RepoKind::exception(SetShortNameAlreadyExist::new(
                                set.short_name(),
                                err.to_string(),
                            ));
                        }
                    }
                }

                RepoKind::unexpected(err)
            })
    }

    async fn delete_by_short_name<'a>(
        &'a mut self,
        set: DeleteByShortName<'a>,
    ) -> Result<(), RepoKind<SetShortNameNotExist<'a>>> {
        let (sql_query, values) = Query::delete()
            .from_table(Alias::new("sets"))
            .and_where(Expr::col(Alias::new("short_name")).eq(set.short_name()))
            .build_sqlx(PostgresQueryBuilder);

        debug!("SQL query: {sql_query};\nValues for query: {values:?}");

        sqlx::query_with(&sql_query, values)
            .execute(&mut *self.conn)
            .await
            .map(|_| ())
            .map_err(|err| {
                if let sqlx::Error::RowNotFound = err {
                    return RepoKind::exception(SetShortNameNotExist::new(
                        set.short_name(),
                        err.to_string(),
                    ));
                }

                RepoKind::unexpected(err)
            })
    }

    async fn get_by_tg_id(
        &mut self,
        set: GetByTgID,
    ) -> Result<Vec<Set>, RepoKind<SetTgIdNotExist>> {
        let (sql_query, values) = Query::select()
            .columns([
                Alias::new("tg_id"),
                Alias::new("short_name"),
                Alias::new("title"),
            ])
            .from(Alias::new("sets"))
            .and_where(Expr::col(Alias::new("tg_id")).eq(set.tg_id()))
            .build_sqlx(PostgresQueryBuilder);

        debug!("SQL query: {sql_query};\nValues for query: {values:?}");

        sqlx::query_as_with(&sql_query, values)
            .fetch_all(&mut *self.conn)
            .await
            .map(|set_model: Vec<SetModel>| set_model.into_iter().map(Into::into).collect())
            .map_err(|err| {
                if let sqlx::Error::RowNotFound = err {
                    return RepoKind::exception(SetTgIdNotExist::new(set.tg_id(), err.to_string()));
                }

                RepoKind::unexpected(err)
            })
    }

    async fn get_one_by_short_name<'a>(
        &'a mut self,
        set: GetByShortName<'a>,
    ) -> Result<Set, RepoKind<SetShortNameNotExist<'a>>> {
        let (sql_query, values) = Query::select()
            .columns([
                Alias::new("tg_id"),
                Alias::new("short_name"),
                Alias::new("title"),
            ])
            .from(Alias::new("sets"))
            .and_where(Expr::col(Alias::new("short_name")).eq(set.short_name()))
            .build_sqlx(PostgresQueryBuilder);

        debug!("SQL query: {sql_query};\nValues for query: {values:?}");

        sqlx::query_as_with(&sql_query, values)
            .fetch_one(&mut *self.conn)
            .await
            .map(|set_model: SetModel| set_model.into())
            .map_err(|err| {
                if let sqlx::Error::RowNotFound = err {
                    return RepoKind::exception(SetShortNameNotExist::new(
                        set.short_name(),
                        err.to_string(),
                    ));
                }

                RepoKind::unexpected(err)
            })
    }
}
