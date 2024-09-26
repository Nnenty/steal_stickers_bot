use std::time::Duration;

use telers::{
    enums::ParseMode,
    errors::session::ErrorKind,
    event::{telegram::HandlerResult, EventReturn},
    fsm::{Context, Storage},
    methods::{
        AddStickerToSet, CreateNewStickerSet, DeleteMessage, GetMe, GetStickerSet, SendMessage,
    },
    types::{InputFile, InputSticker, MessageSticker, MessageText},
    utils::text::{html_bold, html_code},
    Bot,
};
use tracing::{debug, error};

use crate::bot_commands::states::StealStickerSetState;
use crate::{
    common::{generate_sticker_set_name_and_link, sticker_format},
    texts::sticker_set_message,
};

pub async fn steal_sticker_set<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    fsm.finish().await.map_err(Into::into)?;

    fsm.set_state(StealStickerSetState::StealStickerSetName)
        .await
        .map_err(Into::into)?;

    bot.send(SendMessage::new(
        message.chat.id(),
        "Send me a sticker and i will steal this sticker pack for you!",
    ))
    .await?;

    Ok(EventReturn::Finish)
}

pub async fn steal_sticker_set_name<S: Storage>(
    bot: Bot,
    message: MessageSticker,
    fsm: Context<S>,
) -> HandlerResult {
    let set_name = match message.sticker.set_name {
        Some(sticker_set_name) => sticker_set_name,
        None => {
            bot.send(SendMessage::new(
                message.chat.id(),
                "This sticker is without sticker pack! Try to send another sticker pack.",
            ))
            .await?;

            return Ok(EventReturn::Finish);
        }
    };

    fsm.set_value("steal_sticker_set_name", set_name.as_ref())
        .await
        .map_err(Into::into)?;

    fsm.set_state(StealStickerSetState::CreateNewStickerSet)
        .await
        .map_err(Into::into)?;

    bot.send(SendMessage::new(
        message.chat.id(),
        "Now enter name for your new sticker pack (1-64 characters).",
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
            "Too long name for sticker pack! Try enter a name up to 64 characters long.",
        ))
        .await?;

        return Ok(EventReturn::Finish);
    } else if message.text.len() < 1 {
        bot.send(SendMessage::new(
            message.chat.id(),
            "Too short name! Try enter a name between 1 and 64 characters long.",
        ))
        .await?;

        return Ok(EventReturn::Finish);
    } else {
        message.text
    };

    // only panic if i'm forget call fsm.set_value() in function steal_sticker_set_name()
    let steal_sticker_set_name: Box<str> = fsm
        .get_value("steal_sticker_set_name")
        .await
        .map_err(Into::into)?
        .expect("Sticker set name for sticker set user want steal should be set");

    fsm.finish().await.map_err(Into::into)?;

    let steal_sticker_set = bot
        .send(GetStickerSet::new(steal_sticker_set_name.as_ref()))
        .await?;

    let steal_sticker_set_title = steal_sticker_set.title;

    let steal_stickers_from_sticker_set = steal_sticker_set.stickers;

    // cant panic because bot cant be without username
    let bot_username = bot
        .send(GetMe::new())
        .await?
        .username
        .expect("bot without username :/");

    // only panic if bot using in channels, but i'm using private filter in main function
    let user_id = message.from.expect("user without id").id;

    // prepare name for new sticker set and link to use it in message later
    let (mut set_name, mut set_link) = generate_sticker_set_name_and_link(11, &bot_username);

    let message_delete = bot.send(SendMessage::new(
        message.chat.id(),
        format!(
            "Stealing sticker pack with name `{}` for you..\n(stealing sticker packs \
            containing more than 50 stickers can take up to a several minutes due to some internal limitations)",
            set_title
        ),
    ))
    .await?;

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
            user_id,
            &set_name,
            set_title.as_ref(),
            steal_stickers_from_sticker_set[..sticker_set_length]
                .into_iter()
                .map(|sticker| {
                    let sticker_is: InputSticker = InputSticker::new(
                        InputFile::id(sticker.file_id.as_ref()),
                        &sticker_format(&steal_stickers_from_sticker_set)
                            // i explicitly ask the user to send me a sticker, so that the
                            // sticker set will contain at least 1 sticker
                            .expect("empty sticker set"),
                    );

                    sticker_is.emoji_list(sticker.emoji.clone())
                }),
        ))
        .await
    {
        match err {
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

                return Ok(EventReturn::Finish);
            }
        }
    }

    tokio::time::sleep(Duration::from_millis(2111)).await;

    if more_than_50 {
        for sticker in &steal_stickers_from_sticker_set[50..] {
            if let Err(err) = bot
                .send(AddStickerToSet::new(user_id, &set_name, {
                    let sticker_is = InputSticker::new(
                        InputFile::id(sticker.file_id.as_ref()),
                        &sticker_format(&steal_stickers_from_sticker_set)
                            // i explicitly ask the user to send me a sticker, so that the
                            // sticker set will contain at least 1 sticker
                            .expect("empty sticker set"),
                    );

                    sticker_is.emoji_list(sticker.clone().emoji)
                }))
                .await
            {
                error!("error occureded while adding remaining stickers: {}", err);

                bot.send(SendMessage::new(
                    message.chat.id(),
                    format!(
                        "Error occurded while creating new sticker pack, {but_created}! \
                        Due to an error, not all stickers may have been stolen :(\n\
                        (you can delete this sticker pack using the /delpack command in official Telegram bot @Stickers. \
                        Name of this sticker pack: {set_name_code})",
                        but_created = html_bold("but sticker pack was created"),
                        set_name_code = html_code(&set_name)
                    ),
                ).parse_mode(ParseMode::HTML))
                .await?;

                return Ok(EventReturn::Finish);
            }

            // sleep because you can’t send telegram api requests more often than per second
            tokio::time::sleep(Duration::from_millis(1010)).await;
        }
    };

    let steal_sticker_set_link = format!("t.me/addstickers/{}", steal_sticker_set_name);

    let send_sticker_set = sticker_set_message(
        &set_title,
        &set_name,
        &set_link,
        &steal_sticker_set_title,
        &steal_sticker_set_link,
    );

    bot.send(SendMessage::new(message.chat.id(), send_sticker_set).parse_mode(ParseMode::HTML))
        .await?;

    // delete unnecessary message
    bot.send(DeleteMessage::new(
        message_delete.chat().id(),
        message_delete.id(),
    ))
    .await?;

    Ok(EventReturn::Finish)
}
