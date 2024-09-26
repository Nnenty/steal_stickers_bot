use std::borrow::Cow;

use crate::application::common::exceptions::ApplicationException;

#[derive(Debug, thiserror::Error)]
#[error("sticker set with short name `{short_name}` already exists: {message}")]
pub struct SetShortNameAlreadyExist {
    short_name: String,
    message: Cow<'static, str>,
}

impl SetShortNameAlreadyExist {
    pub fn new(short_name: String, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            short_name,
            message: message.into(),
        }
    }
}

impl ApplicationException for SetShortNameAlreadyExist {}

#[derive(Debug, thiserror::Error)]
#[error("sticker set with short name `{short_name}` not exists: {message}")]
pub struct SetShortNameNotExist {
    short_name: String,
    message: Cow<'static, str>,
}

impl SetShortNameNotExist {
    pub fn new(short_name: String, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            short_name,
            message: message.into(),
        }
    }
}

impl ApplicationException for SetShortNameNotExist {}

#[derive(Debug, thiserror::Error)]
#[error("sticker sets with Telegram ID `{tg_id}` not exists: {message}")]
pub struct SetTgIdNotExist {
    tg_id: i64,
    message: Cow<'static, str>,
}

impl SetTgIdNotExist {
    pub fn new(tg_id: i64, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            tg_id,
            message: message.into(),
        }
    }
}

impl ApplicationException for SetTgIdNotExist {}
