use grammers_client::{
    client::bots::{AuthorizationError, InvocationError},
    SignInError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // grammers errors
    #[error(transparent)]
    AuthorizationError(#[from] AuthorizationError),
    #[error(transparent)]
    InvocationError(#[from] InvocationError),
    #[error(transparent)]
    SignInError(#[from] SignInError),

    // other
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Std(#[from] std::io::Error),
}
