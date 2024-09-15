use async_trait::async_trait;

use crate::{application::common::exceptions::RepoKind, entities::set::Set};

use super::{
    dto::{
        create::Create, delete_by_short_name::DeleteByShortName, get_by_short_name::GetByShortName,
        get_by_tg_id::GetByTgID,
    },
    exceptions::{SetShortNameAlreadyExist, SetShortNameNotExist, SetTgIdNotExist},
};

#[async_trait]
pub trait SetRepo {
    async fn create(&mut self, set: Create) -> Result<(), RepoKind<SetShortNameAlreadyExist>>;
    async fn delete_by_short_name(
        &mut self,
        set: DeleteByShortName,
    ) -> Result<(), RepoKind<SetShortNameNotExist>>;

    async fn get_by_tg_id(&mut self, set: GetByTgID) -> Result<Set, RepoKind<SetTgIdNotExist>>;
    async fn get_by_short_name(
        &mut self,
        set: GetByShortName,
    ) -> Result<Set, RepoKind<SetShortNameNotExist>>;
}
