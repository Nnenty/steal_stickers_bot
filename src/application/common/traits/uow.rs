use async_trait::async_trait;

use crate::application::{
    common::exceptions::{BeginError, CommitError, RollbackError},
    set::traits::SetRepo,
    user::traits::UserRepo,
};

#[async_trait]
pub trait UoW {
    type Connection<'a>
    where
        Self: 'a;

    async fn connect(&mut self) -> Result<Self::Connection<'_>, BeginError>;

    async fn begin(&mut self) -> Result<(), BeginError>;

    async fn commit(&mut self) -> Result<(), CommitError>;

    async fn rollback(&mut self) -> Result<(), RollbackError>;

    async fn user_repo(&mut self) -> Result<impl UserRepo, BeginError>;

    async fn set_repo(&mut self) -> Result<impl SetRepo, BeginError>;
}

pub trait UoWFactory {
    type UoW;

    fn create_uow(&self) -> Self::UoW;
}
