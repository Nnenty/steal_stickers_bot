use std::borrow::Cow;

use crate::application::common::exceptions::ApplicationException;

#[derive(Debug, thiserror::Error)]
#[error("user with telegram ID `{tg_id}` already exists: {message}")]
pub struct UserTgIdAlreadyExists {
    tg_id: i64,
    message: Cow<'static, str>,
}

impl UserTgIdAlreadyExists {
    pub fn new(tg_id: i64, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            tg_id,
            message: message.into(),
        }
    }
}

impl ApplicationException for UserTgIdAlreadyExists {}

#[derive(Debug, thiserror::Error)]
#[error("user with telegram ID `{tg_id}` not exists: {message}")]
pub struct UserTgIdNotExist {
    tg_id: i64,
    message: Cow<'static, str>,
}

impl UserTgIdNotExist {
    pub fn new(tg_id: i64, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            tg_id,
            message: message.into(),
        }
    }
}

impl ApplicationException for UserTgIdNotExist {}
