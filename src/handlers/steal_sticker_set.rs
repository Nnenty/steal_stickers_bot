use std::time::Duration;

use futures::StreamExt;
use telers::{
    enums::ParseMode,
    errors::session::ErrorKind,
    event::{telegram::HandlerResult, EventReturn},
    fsm::{Context, Storage},
    methods::{AddStickerToSet, CreateNewStickerSet, GetMe, GetStickerSet, SendMessage},
    types::{InputFile, InputSticker, Message, MessageSticker, MessageText},
    Bot,
};
use tracing::{debug, error, warn};

use crate::core::{generate_sticker_set_name_and_link, send_sticker_set_message, sticker_format};
use crate::State;

pub async fn steal_handler<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    bot.send(SendMessage::new(
        message.chat.id(),
        "Send me a sticker and I will steal a sticker pack containing that sticker for you!",
    ))
    .await?;

    fsm.set_state(State::StealStickerSetName)
        .await
        .map_err(Into::into)?;

    Ok(EventReturn::Finish)
}

pub async fn steal_sticker_set_handler<S: Storage>(
    bot: Bot,
    message: MessageSticker,
    fsm: Context<S>,
) -> HandlerResult {
    let set_name = message.sticker.set_name;

    fsm.set_value("steal_sticker_set_name", set_name.clone())
        .await
        .map_err(Into::into)?;

    fsm.set_state(State::CreateNewStickerSet)
        .await
        .map_err(Into::into)?;

    bot.send(SendMessage::new(
        message.chat.id(),
        format!("Then enter name for your new sticker pack (1-64 characters):"),
    ))
    .await?;

    Ok(EventReturn::Finish)
}

pub async fn process_wrong_sticker(bot: Bot, message: Message) -> HandlerResult {
    bot.send(SendMessage::new(
        message.chat().id(),
        "Please, send me a sticker.",
    ))
    .await?;

    Ok(EventReturn::Finish)
}

/// ### Panics
/// - Panics if user is unknown (only if message sent in channel)
pub async fn create_new_sticker_set<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    // if user enter wrong sticker set title, process it
    let set_title = if message.text.len() > 64 {
        bot.send(SendMessage::new(
            message.chat.id(),
            "Too long name for sticker pack!\nTry enter a name up to 64 characters long.",
        ))
        .await?;

        return Ok(EventReturn::Cancel);
    } else if message.text.len() < 1 {
        bot.send(SendMessage::new(
            message.chat.id(),
            "Too short name!\nTry enter a name between 1 and 64 characters long.",
        ))
        .await?;

        return Ok(EventReturn::Cancel);
    } else {
        message.text
    };

    let steal_sticker_set_name: Box<str> = match fsm
        .get_value("steal_sticker_set_name")
        .await
        .map_err(Into::into)?
        .expect("Sticker set name for sticker set user want steal should be set")
    {
        Some(sssn) => sssn,

        None => {
            error!("An error occurds while parsing name of this sticker pack: name is empty.");

            bot.send(SendMessage::new(
                message.chat.id(),
                "An error occurds while parsing name of this sticker pack: name is empty;\nTry again.",
            ))
            .await?;

            return Ok(EventReturn::Cancel);
        }
    };

    let steal_sticker_set = bot
        .send(GetStickerSet::new(steal_sticker_set_name.as_ref()))
        .await?;

    let steal_sticker_set_title = steal_sticker_set.title;

    let steal_stickers_from_sticker_set = steal_sticker_set.stickers;

    // finish is not at the end because if an error occurs, the state will not be cleared
    fsm.finish().await.map_err(Into::into)?;

    // prepare info for new sticker set
    let bot_username = bot
        .send(GetMe::new())
        .await?
        .username
        .expect("bot without username :/");

    let user = message.from.expect("error while parsing user");

    // prepare name for new sticker set and link to use it in message later
    let (mut set_name, mut set_link) = generate_sticker_set_name_and_link(11, &bot_username);

    // prepare sticker format for new sticker set
    let sticker_format = match sticker_format(&steal_stickers_from_sticker_set) {
        Some(first_sticker) => first_sticker,

        None => {
            fsm.finish().await.map_err(Into::into)?;

            error!("empty sticker pack to copy");

            bot.send(SendMessage::new(
                message.chat.id(),
                "Sticker pack that you want to copy is empty. Please, try to send another pack!",
            ))
            .await?;

            return Ok(EventReturn::Cancel);
        }
    };

    bot.send(SendMessage::new(
        message.chat.id(),
        format!(
            "Creating sticker pack with name `{}` for you..\n(creating sticker packs \
            containing more than 50 stickers can take up to a minute)",
            set_title
        ),
    ))
    .await?;

    debug!("set name: {} set title: {}", set_name, set_title);

    let (steal_stickers_from_sticker_set, sticker_set_length, more_than_50) =
        if steal_stickers_from_sticker_set.len() > 50 {
            (Box::new(&steal_stickers_from_sticker_set[..]), 50, true)
        } else {
            (
                Box::new(&steal_stickers_from_sticker_set[..]),
                steal_stickers_from_sticker_set.len(),
                false,
            )
        };

    while let Err(err) = bot
        .send(CreateNewStickerSet::new(
            user.id,
            &set_name,
            set_title.as_ref(),
            steal_stickers_from_sticker_set[..sticker_set_length]
                .into_iter()
                .map(|sticker| {
                    let sticker_is =
                        InputSticker::new(InputFile::id(sticker.file_id.as_ref()), sticker_format);

                    sticker_is.emoji_list(sticker.clone().emoji)
                }),
        ))
        .await
    {
        match err {
            // if generated name is invalid or sticker set with this name already exists, regenerate it
            ErrorKind::Telegram(err) => {
                if err.to_string()
                    == r#"TelegramBadRequest: "Bad Request: sticker set name is already occupied""#
                {
                    error!(
                        "file to create new sticker set: {}; try generate sticker set name again..",
                        err
                    );
                    debug!("sticker set name: {}", set_name);

                    (set_name, set_link) = generate_sticker_set_name_and_link(11, &bot_username);
                } else {
                    error!("file to create new sticker set: {}", err);
                    debug!("sticker set name: {}", set_name);

                    bot.send(SendMessage::new(
                        message.chat.id(),
                        format!("Error occurded while creating new sticker pack :(\nTry again."),
                    ))
                    .await?;

                    return Ok(EventReturn::Cancel);
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
    if more_than_50 {
        for sticker in &steal_stickers_from_sticker_set[50..] {
            tokio::time::sleep(Duration::from_millis(1100)).await;

            bot.send(AddStickerToSet::new(user.id, &set_name, {
                let sticker_is =
                    InputSticker::new(InputFile::id(sticker.file_id.as_ref()), sticker_format);

                sticker_is.emoji_list(sticker.clone().emoji)
            }))
            .await?;
        }
    };

    let steal_sticker_set_link = format!("t.me/addstickers/{}", steal_sticker_set_name);

    let send_sticker_set = send_sticker_set_message(
        &set_title,
        &set_name,
        &set_link,
        &steal_sticker_set_title,
        &steal_sticker_set_link,
    );

    bot.send(SendMessage::new(message.chat.id(), send_sticker_set).parse_mode(ParseMode::HTML))
        .await?;

    Ok(EventReturn::Finish)
}
