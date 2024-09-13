use std::borrow::Cow;

pub trait ApplicationException {}
pub trait UnexpectedError {}

#[derive(Debug, thiserror::Error)]
#[error("repository error: {message}")]
pub struct RepoError {
    message: Cow<'static, str>,
}

impl ApplicationException for RepoError {}
impl UnexpectedError for RepoError {}

impl RepoError {
    pub fn new(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RepoKind<RepoException>
where
    RepoException: ApplicationException,
{
    #[error(transparent)]
    Exception(RepoException),
    #[error(transparent)]
    Unexpected(RepoError),
}

impl<RepoException> RepoKind<RepoException>
where
    RepoException: ApplicationException,
{
    pub fn exception(exception: impl Into<RepoException>) -> Self {
        Self::Exception(exception.into())
    }

    pub fn unexpected(error: impl Into<RepoError>) -> Self {
        Self::Unexpected(error.into())
    }
}
