use crate::application::common::exceptions::{ApplicationException, RepoError, RepoKind};

pub mod user;

impl From<sqlx::Error> for RepoError {
    fn from(error: sqlx::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl<RepoException> From<sqlx::Error> for RepoKind<RepoException>
where
    RepoException: ApplicationException,
{
    fn from(error: sqlx::Error) -> Self {
        Self::unexpected(error)
    }
}
