use telers::{
    enums::ParseMode,
    errors::session::ErrorKind,
    event::{telegram::HandlerResult, EventReturn},
    fsm::{Context, Storage},
    methods::{AddStickerToSet, GetStickerSet, SendMessage},
    types::{InputFile, InputSticker, MessageSticker, MessageText},
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
    fsm.finish().await.map_err(Into::into)?;

    bot.send(SendMessage::new(
        message.chat.id(),
        "Send me your sticker pack that stolen by this bot!\n\
        (if you don't have the sticker packs stolen by this bot, first use the command /steal_pack)",
    ))
    .await?;

    fsm.set_state(AddStickerState::GetStolenStickerSet)
        .await
        .map_err(Into::into)?;

    Ok(EventReturn::Finish)
}

pub async fn get_stolen_sticker_set<S: Storage>(
    bot: Bot,
    message: MessageSticker,
    fsm: Context<S>,
) -> HandlerResult {
    let sticker_set_name = match message.sticker.set_name {
        Some(sticker_set_name) => sticker_set_name,
        None => {
            bot.send(SendMessage::new(
                message.chat.id(),
                "This sticker is without the sticker pack! Try sending another sticker pack.",
            ))
            .await?;

            return Ok(EventReturn::Finish);
        }
    };

    debug!("1 {sticker_set_name}");

    fsm.set_value("get_stolen_sticker_set", sticker_set_name)
        .await
        .map_err(Into::into)?;

    fsm.set_state(AddStickerState::AddStickerToStolenStickerSet)
        .await
        .map_err(Into::into)?;

    bot.send(SendMessage::new(
        message.chat.id(),
        "Now send me any sticker you want to add in stolen sticker pack:",
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
    // only panic if i'm forget call fsm.set_value() in function get_stolen_sticker_set()
    let sticker_set_name: Box<str> = fsm
        .get_value("get_stolen_sticker_set")
        .await
        .map_err(Into::into)?
        .expect("Sticker set name for sticker set user want steal should be set");

    debug!("2 {sticker_set_name}");

    fsm.finish().await.map_err(Into::into)?;

    let sticker_to_add = message.sticker;

    // i'm explicitly passing the sticker so the slice can't be empty
    let sticker_format = sticker_format(&[sticker_to_add.clone()]).expect("stickers not specifed");

    // only panic if messages uses in channels, but i'm using private filter in main function
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
            ErrorKind::Telegram(err) => {
                if err.to_string() == r#"TelegramBadRequest: "Bad Request: STICKERSET_INVALID""# {
                    error!("file to add sticker: {}", err);
                    debug!("sticker set name: {}", sticker_set_name);

                    bot.send(SendMessage::new(
                        message.chat.id(),
                        "Error! I didn't steal this sticker pack!\nTry calling /steal_pack and send me stolen sticker pack.",
                    ))
                    .await?;

                    return Ok(EventReturn::Finish);
                } else {
                    error!("file to add sticker: {}", err);
                    debug!("sticker set name: {}", sticker_set_name);

                    bot.send(SendMessage::new(
                        message.chat.id(),
                        "Error occurded while adding sticker to sticker pack :(\nTry again.",
                    ))
                    .await?;

                    return Ok(EventReturn::Finish);
                }
            }
            err => {
                error!(
                    "error occureded while adding sticker sticker set: {}\n",
                    err
                );
                debug!("sticker set name: {}", sticker_set_name);

                bot.send(SendMessage::new(
                    message.chat.id(),
                    "Error occurded while adding sticker to sticker pack :(",
                ))
                .await?;

                return Ok(EventReturn::Finish);
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
