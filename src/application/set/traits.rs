use async_trait::async_trait;

use crate::{
    application::common::exceptions::{RepoError, RepoKind},
    entities::set::Set,
};

use super::{
    dto::{
        create::Create, delete_by_short_name::DeleteByShortName, get_by_short_name::GetByShortName,
        get_by_tg_id::GetByTgID,
    },
    exceptions::{SetShortNameAlreadyExist, SetShortNameNotExist},
};

#[async_trait]
pub trait SetRepo {
    async fn create<'a>(
        &'a mut self,
        set: Create<'a>,
    ) -> Result<(), RepoKind<SetShortNameAlreadyExist>>;
    async fn delete_by_short_name<'a>(
        &'a mut self,
        set: DeleteByShortName<'a>,
    ) -> Result<(), RepoKind<SetShortNameNotExist>>;

    async fn get_by_tg_id(&mut self, set: GetByTgID) -> Result<Vec<Set>, RepoError>;
    async fn get_by_short_name<'a>(
        &'a mut self,
        set: GetByShortName<'a>,
    ) -> Result<Set, RepoKind<SetShortNameNotExist>>;
}
