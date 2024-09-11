use std::{io, time::Duration};

use grammers_client::{Client, Config, FixedReconnect, InitParams, SignInError};
use grammers_session::Session;
use grammers_tl_types::{
    enums::{self, InputStickerSet},
    functions::messages::{GetAllStickers, GetStickerSet},
    types::{self, InputStickerSetShortName},
};

use tracing::error;

mod constants;
mod errors;
use constants::SESSION_FILE;

static RECONNECT_POLICY: FixedReconnect = FixedReconnect {
    attempts: 3,
    delay: Duration::from_millis(100),
};

pub async fn client_connect(api_id: i32, api_hash: String) -> Result<Client, errors::Error> {
    Ok(Client::connect(Config {
        session: Session::load_file_or_create(SESSION_FILE)?,
        api_id,
        api_hash,
        params: InitParams {
            reconnection_policy: &RECONNECT_POLICY,
            ..Default::default()
        },
    })
    .await?)
}

pub async fn client_authorize(
    client: &Client,
    phone: &str,
    password: &str,
) -> Result<(), errors::Error> {
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
        println!("Signed in!");

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

pub async fn get_owned_stolen_sticker_sets(
    client: &Client,
    user_id: i64,
    bot_username: &str,
) -> Result<Vec<(String, String)>, errors::Error> {
    let sets: Vec<enums::StickerSet> =
        match client.invoke(&GetAllStickers { hash: user_id }).await? {
            enums::messages::AllStickers::Stickers(
                types::messages::AllStickers { sets, .. },
                ..,
            ) => sets,
            _ => todo!(),
        };

    let mut sets_names: Vec<(String, String)> = Vec::new();

    sets.into_iter()
        .filter(|set| {
            let enums::StickerSet::Set(types::StickerSet {
                short_name,
                creator,
                ..
            }) = set;

            creator.to_owned() && short_name.contains(format!("by_{bot_username}").as_str())
        })
        .for_each(|set| {
            let enums::StickerSet::Set(types::StickerSet {
                short_name, title, ..
            }) = set;

            sets_names.push((short_name, title));
        });

    Ok(sets_names)
}

pub async fn get_sticker_set_user_id(
    set_name: &str,
    client: &Client,
) -> Result<i64, errors::Error> {
    let set_id = match client
        .invoke(&GetStickerSet {
            stickerset: InputStickerSet::ShortName(InputStickerSetShortName {
                short_name: set_name.to_owned(),
            }),
            hash: 0,
        })
        .await?
    {
        enums::messages::StickerSet::Set(types::messages::StickerSet {
            set: enums::StickerSet::Set(types::StickerSet { id, .. }),
            ..
        }) => id,
        _ => todo!(),
    };

    let mut user_id = set_id >> 32;
    if set_id >> 24 & 0xff == 1 {
        user_id += 0x100000000
    }

    Ok(user_id)
}
