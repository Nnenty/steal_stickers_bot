use grammers_client::{Client, SignInError};
use std::io;
use toml;
use tracing::{debug, error};

pub mod constants;
mod errors;
use crate::ConfigToml;
pub use constants::SESSION_FILE;

pub async fn authorize(client: Client, config_toml_path: &str) -> Result<(), errors::Error> {
    let config = std::fs::read_to_string(config_toml_path)?;
    let ConfigToml { auth, .. } = toml::from_str(&config)?;

    let phone = auth.phone_number;

    let mut sign_out = false;

    if !client.is_authorized().await? {
        let token = client.request_login_code(&phone).await?;

        debug!("Enter the code you received:");
        let mut code = String::new();
        io::stdin().read_line(&mut code)?;
        let code = code.trim();

        match client.sign_in(&token, code).await {
            Err(SignInError::PasswordRequired(password_token)) => {
                let password = auth.password;

                client
                    .check_password(password_token, password.trim())
                    .await?;
            }
            Ok(_) => (),
            Err(err) => return Err(err.into()),
        };
        debug!("Signed in!");

        match client.session().save_to_file(SESSION_FILE) {
            Ok(_) => {}
            Err(e) => {
                error!("NOTE: failed to save the session, will sign out when done: {e}");
                sign_out = true;
            }
        }
    }

    if sign_out {
        // TODO revisit examples and get rid of "handle references" (also, this panics)
        drop(client.sign_out_disconnect().await);
    }

    Ok(())
}
