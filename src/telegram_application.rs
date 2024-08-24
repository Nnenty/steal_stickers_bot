use std::io;
use tracing::{debug, error};

use grammers_client::{client::bots::AuthorizationError, Client, Config, SignInError};
use grammers_session::Session;

mod constants;
mod errors;
use constants::SESSION_FILE;

pub async fn client_connect(api_id: i32, api_hash: String) -> Result<Client, AuthorizationError> {
    Ok(Client::connect(Config {
        session: Session::load_file_or_create(SESSION_FILE)?,
        api_id,
        api_hash,
        params: Default::default(),
    })
    .await?)
}

pub async fn authorize(client: &Client, phone: &str, password: &str) -> Result<(), errors::Error> {
    let mut sign_out = false;

    if !client.is_authorized().await? {
        let token = client.request_login_code(&phone).await?;

        println!("Enter the code you received on your Telegram account:");
        let mut code = String::new();
        io::stdin().read_line(&mut code)?;
        let code = code.trim();

        match client.sign_in(&token, code).await {
            Err(SignInError::PasswordRequired(password_token)) => {
                let password = password;

                client
                    .check_password(password_token, password.trim())
                    .await?;
            }
            Ok(_) => (),
            Err(err) => return Err(err.into()),
        };
        debug!("signed in!");

        match client.session().save_to_file(SESSION_FILE) {
            Ok(_) => {}
            Err(e) => {
                error!("NOTE: failed to save the session, will sign out when done: {e}");
                sign_out = true;
            }
        }
    }

    if sign_out {
        drop(client.sign_out_disconnect().await);
    }

    Ok(())
}
