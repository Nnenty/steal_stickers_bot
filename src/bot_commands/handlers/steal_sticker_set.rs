use telers::{
    enums::ParseMode,
    errors::session::ErrorKind,
    event::{telegram::HandlerResult, EventReturn},
    fsm::{Context, Storage},
    methods::{CreateNewStickerSet, DeleteMessage, GetMe, GetStickerSet, SendMessage},
    types::{InputFile, InputSticker, MessageSticker, MessageText},
    utils::text::{html_bold, html_code},
    Bot,
};
use tracing::error;

use crate::{
    bot_commands::states::StealStickerSetState,
    core::stickers::constants::CREATE_SET_IN_ONE_GO_LENGTH_LIMIT,
};
use crate::{
    common::{generate_sticker_set_name_and_link, sticker_format},
    texts::sticker_set_message,
};

use super::common::add_stickers;

pub async fn steal_sticker_set_handler<S: Storage>(
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
            "Stealing sticker pack with name `{set_title}` for you..\n(stealing sticker packs \
            containing more than {CREATE_SET_IN_ONE_GO_LENGTH_LIMIT} stickers can take up to a several minutes due to some internal limitations)",
        ),
    ))
    .await?;

    let (limit_sticker_set_length, more_than_limit) =
        if steal_stickers_from_sticker_set.len() > CREATE_SET_IN_ONE_GO_LENGTH_LIMIT {
            (CREATE_SET_IN_ONE_GO_LENGTH_LIMIT, true)
        } else {
            (steal_stickers_from_sticker_set.len(), false)
        };

    while let Err(err) = bot
        .send(CreateNewStickerSet::new(
            user_id,
            &set_name,
            set_title.as_ref(),
            steal_stickers_from_sticker_set[..limit_sticker_set_length]
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
                        ?err,
                        "file to create new sticker set; try generate sticker set name again:"
                    );
                    error!(set_name, "sticker set name:");

                    (set_name, set_link) = generate_sticker_set_name_and_link(11, &bot_username);
                } else {
                    error!(?err, "file to create new sticker set:");
                    error!(set_name, "sticker set name:");

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

    if more_than_limit {
        let all_stickers_was_added = add_stickers(
            &bot,
            user_id,
            set_name.as_ref(),
            steal_stickers_from_sticker_set[limit_sticker_set_length..].as_ref(),
        )
        .await
        .expect("empty stickers list");

        if !all_stickers_was_added {
            bot.send(SendMessage::new(
                message.chat.id(),
                format!(
                    "Error occurded while creating new sticker pack, {but_created}! \
                    Due to an error, not all stickers have been stolen :(\n\
                    (you can delete this sticker pack using the /delpack command in official Telegram bot @Stickers. \
                    Name of this sticker pack: {copy_set_name})",
                    but_created = html_bold("but sticker pack was created"),
                    copy_set_name = html_code(&set_name)
                ),
            ).parse_mode(ParseMode::HTML))
            .await?;

            return Ok(EventReturn::Finish);
        }
    }

    let steal_sticker_set_link = format!("t.me/addstickers/{}", steal_sticker_set_name);

    bot.send(
        SendMessage::new(
            message.chat.id(),
            sticker_set_message(
                &set_title,
                &set_name,
                &set_link,
                &steal_sticker_set_title,
                &steal_sticker_set_link,
            ),
        )
        .parse_mode(ParseMode::HTML),
    )
    .await?;

    // delete unnecessary message
    bot.send(DeleteMessage::new(
        message_delete.chat().id(),
        message_delete.id(),
    ))
    .await?;

    Ok(EventReturn::Finish)
}
