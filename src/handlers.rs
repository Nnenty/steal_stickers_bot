use random_string::generate;
use telers::{
    enums::ParseMode,
    errors::session::ErrorKind,
    event::{telegram::HandlerResult, EventReturn},
    fsm::{Context, Storage},
    methods::{CreateNewStickerSet, GetMe, GetStickerSet, SendMessage},
    types::{InputFile, InputSticker, Message, MessageSticker, MessageText},
    utils::text::html_text_link,
    Bot,
};
use tracing::{debug, error};

use crate::states;
use states::State;

// DELETE `// dev` IN FUNCTIONS:
// steal_sticker_set_handler
// get_new_sticker_set_title
// create_new_sticker_set

pub async fn start_handler<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    bot.send(SendMessage::new(
        message.chat.id(),
        "
    Hello! This is bot to steal stickers.\n\
    List of commands you can use:
    /help - Show help message 
    /steal - Steal sticker pack
    /cancel - Cancel last command
\n\
    (you can't steal stickers in channels)
        ",
    ))
    .await?;

    fsm.finish().await.map_err(Into::into)?;

    Ok(EventReturn::Finish)
}

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

    // set a new state to get the name for the new sticker set
    fsm.set_state(State::CreateNewStickerSet)
        .await
        .map_err(Into::into)?;

    // after this message should call `get_new_stickerset_name()` function
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
        "Please send me a sticker.",
    ))
    .await?;

    Ok(EventReturn::Finish)
}

pub async fn cancel_handler<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    fsm.finish().await.map_err(Into::into)?;

    bot.send(SendMessage::new(
        message.chat.id(),
        "Last command was canceled.",
    ))
    .await?;

    Ok(EventReturn::Cancel)
}

/// # Panics
/// Panics if user is unknown (only if message sent in channel)
pub async fn create_new_sticker_set<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    let steal_sticker_set_name: Box<str> = match fsm
        .get_value("steal_sticker_set_name")
        .await
        .map_err(Into::into)?
        .expect("Sticker set name for sticker set you want steal should be set")
    {
        Some(sssn) => sssn,

        None => {
            error!("Error occurded while parsing name of this sticker pack: name is empty.");

            bot.send(SendMessage::new(
                message.chat.id(),
                "Error occurded while parsing name of this sticker pack: name is empty;\nTry again.",
            ))
            .await?;

            return Ok(EventReturn::Cancel);
        }
    };

    let set_title = if message.text.len() > 64 {
        bot.send(SendMessage::new(
            message.chat.id(),
            "Too long name for sticker pack;\nthe name has been shortened to 64 characters.",
        ))
        .await?;

        message.text[..64].to_string().into_boxed_str()
    } else {
        message.text
    };

    // finish is not at the end because if an error occurs, the state will not be cleared
    fsm.finish().await.map_err(Into::into)?;

    let steal_stickers_from_sticker_set = bot
        .send(GetStickerSet::new(steal_sticker_set_name.as_ref()))
        .await?
        .stickers;

    // prepare sticker format for new sticker set
    let sticker_format = match steal_stickers_from_sticker_set.iter().next() {
        Some(first_sticker) if first_sticker.is_animated => "animated",
        Some(first_sticker) if first_sticker.is_video => "video",
        Some(_) => "static",

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

    // prepare info for new sticker set
    let bot_user = bot
        .send(GetMe::new())
        .await?
        .username
        .expect("bot without username :/");

    // to generate random name for sticker set
    let charset = "abcg890hijklmJKxyzAnopqrstuvwBefCDEFGHIQRSTUVWXYZ1237LMNOP45d6";

    // prepare name for new sticker set
    let set_name = String::from(generate(11, charset));
    let mut set_name = format!("{}_by_{}", set_name, bot_user);

    let user = message.from.expect("error while parsing user");

    bot.send(SendMessage::new(
        message.chat.id(),
        format!("Creating sticker pack with name `{}` for you..", set_title),
    ))
    .await?;

    while let Err(err) = bot
        .send(CreateNewStickerSet::new(
            user.id,
            &set_name,
            set_title.as_ref(),
            steal_stickers_from_sticker_set.into_iter().map(|sticker| {
                let sticker_is =
                    InputSticker::new(InputFile::id(sticker.file_id.as_ref()), sticker_format);

                sticker_is.emoji_list(sticker.clone().emoji)
            }),
        ))
        .await
    {
        match err {
            ErrorKind::Telegram(err) => {
                if err.to_string()
                    == r#"TelegramBadRequest: "Bad Request: sticker set name is already occupied""#
                    || err.to_string()
                        == r#"TelegramBadRequest: "Bad Request: invalid sticker set name is specified""#
                {
                    error!(
                        "file to create new sticker set: {}; try generate sticker set name again..",
                        err
                    );
                    debug!("sticker set name: {}", set_name);

                    set_name = String::from(generate(11, charset));
                    set_name = format!("{}_by_{}", set_name, bot_user);
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

    let link = format!("t.me/addstickers/{}", set_name);
    let steal_link = format!("t.me/addstickers/{}", steal_sticker_set_name);

    let send_sticker_set = format!(
        "
        Then you have your own sticker pack {new_ss_url}!\n(original {steal_ss_url})\n\nIf you want to check/update your\
        new sticker pack, use official Telegram bot @Stickers, which does an excellent job of managing sticker packs.\n\
        (the name of your new sticker pack will be similar to `spscjnXrbLA_by_steal_stickers_bot`)
\n\
        List of commands in @Stickers that may be useful to you:\n\
        /addsticker – add a sticker to an existing set\n\
        /editsticker – change emoji or coordinates\n\
        /replacesticker – replace stickers in a set\n\
        /ordersticker – reorder stickers in a set\n\
        /delsticker – remove a sticker from an existing set\n\
        /setpackicon – set a sticker set icon\n\
        /renamepack – rename a set\n\
        /delpack – delete a set
        ",
        new_ss_url = html_text_link(&set_title, link),
        steal_ss_url = html_text_link(steal_sticker_set_name, &steal_link),
    );

    bot.send(SendMessage::new(message.chat.id(), send_sticker_set).parse_mode(ParseMode::HTML))
        .await?;

    Ok(EventReturn::Finish)
}
