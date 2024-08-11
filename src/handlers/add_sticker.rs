use telers::{
    enums::ParseMode,
    errors::session::ErrorKind,
    event::{telegram::HandlerResult, EventReturn},
    fsm::{Context, Storage},
    methods::{AddStickerToSet, GetStickerSet, SendMessage},
    types::{InputFile, InputSticker, MessageSticker, MessageText, Sticker},
    utils::text::html_text_link,
    Bot,
};

use tracing::{debug, error};

use crate::core::sticker_format;
use crate::AddStickerState;

pub async fn steal_sticker_handler<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    bot.send(
        SendMessage::new(
            message.chat.id(),
            format!(
                "Send me any sticker that will be added to your sticker pack stolen by this bot!\n\
        (there is no list of your stickers that were stolen by this bot, because I'm too lazy \
        to make a database. If you want a list here, write to {my_profile})",
                my_profile = html_text_link("me", "https://t.me/nnenty")
            ),
        )
        .parse_mode(ParseMode::HTML),
    )
    .await?;

    fsm.set_state(AddStickerState::StealSticker)
        .await
        .map_err(Into::into)?;

    Ok(EventReturn::Finish)
}

pub async fn get_user_owned_sticker_set<S: Storage>(
    bot: Bot,
    message: MessageSticker,
    fsm: Context<S>,
) -> HandlerResult {
    let sticker = message.sticker;

    // bug in telers
    fsm.set_value("steal_sticker", serde_json::to_string(&sticker).unwrap())
        .await
        .map_err(Into::into)?;

    fsm.set_state(AddStickerState::AddStickerToUserOwnedStickerSet)
        .await
        .map_err(Into::into)?;

    bot.send(SendMessage::new(
        message.chat.id(),
        "Now send a sticker from the sticker pack that was stolen by this bot:",
    ))
    .await?;

    Ok(EventReturn::Finish)
}

/// ### Panics
/// - Panics if user is unknown (only if message sent in channel)
pub async fn add_sticker_to_user_owned_sticker_set<S: Storage>(
    bot: Bot,
    message: MessageSticker,
    fsm: Context<S>,
) -> HandlerResult {
    // bug in telers crate
    let sticker_to_add: Sticker = serde_json::from_str::<Sticker>(
        &fsm.get_value::<_, String>("steal_sticker")
            .await
            .map_err(Into::into)?
            .expect("Sticker set name for sticker set user want steal should be set"),
    )
    .unwrap();

    fsm.finish().await.map_err(Into::into)?;

    let sticker_set_name = match message.sticker.set_name {
        Some(sssn) => sssn,

        None => {
            error!("An error occurds while parsing name of this sticker pack: name is empty.");

            bot.send(SendMessage::new(
            message.chat.id(),
            "An error occurds while parsing name of this sticker pack: name is empty;\nTry again.",
        ))
        .await?;

            return Ok(EventReturn::Finish);
        }
    };

    let sticker_format =
        sticker_format(&[sticker_to_add.clone()]).expect("sticker without sticker :/");

    let user_id = message.from.expect("error while parsing user").id;

    bot.send(SendMessage::new(
        message.chat.id(),
        "Try to add sticker to your sticker pack..",
    ))
    .await?;

    if let Err(err) = bot
        .send(AddStickerToSet::new(user_id, sticker_set_name.as_ref(), {
            let sticker_is = InputSticker::new(
                InputFile::id(sticker_to_add.file_id.as_ref()),
                sticker_format,
            );

            sticker_is.emoji_list(sticker_to_add.emoji)
        }))
        .await
    {
        match err {
            // if generated name is invalid or sticker set with this name already exists, regenerate it
            ErrorKind::Telegram(err) => {
                if err.to_string() == r#"TelegramBadRequest: "Bad Request: STICKERSET_INVALID""# {
                    error!("file to add sticker: {}", err);
                    debug!("sticker set name: {}", sticker_set_name);

                    bot.send(SendMessage::new(
                        message.chat.id(),
                        "Error. I didn't steal this sticker pack!\nTry calling /steal_pack and resending this sticker pack again.",
                    ))
                    .await?;

                    return Ok(EventReturn::Finish);
                } else {
                    error!("file to add sticker: {}", err);
                    debug!("sticker set name: {}", sticker_set_name);

                    bot.send(SendMessage::new(
                        message.chat.id(),
                        "Error occurded while creating new sticker pack :(\nTry again.",
                    ))
                    .await?;

                    return Ok(EventReturn::Finish);
                }
            }
            err => {
                error!("error occureded while creating new sticker set: {}\n", err);

                bot.send(SendMessage::new(
                    message.chat.id(),
                    format!("Error occurded while creating new sticker pack :("),
                ))
                .await?;

                return Err(err.into());
            }
        }
    }
    let sticker_set_title = bot
        .send(GetStickerSet::new(sticker_set_name.as_ref()))
        .await?
        .title;

    bot.send(
        SendMessage::new(
            message.chat.id(),
            format!(
                "This sticker was added into {}!",
                html_text_link(
                    sticker_set_title,
                    format!("t.me/addstickers/{}", sticker_set_name)
                )
            ),
        )
        .parse_mode(ParseMode::HTML),
    )
    .await?;

    Ok(EventReturn::Finish)
}
