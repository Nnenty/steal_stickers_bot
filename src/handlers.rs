use random_string::generate;
use telers::{
    enums::ParseMode,
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
        format!("Then enter name for your new sticker pack:"),
    ))
    .await?;

    Ok(EventReturn::Finish)
}

pub async fn process_wrong_sticker(bot: Bot, message: Message) -> HandlerResult {
    bot.send(SendMessage::new(
        message.chat().id(),
        "Please send me a sticker because I only can work with stickers.",
    ))
    .await?;

    Ok(EventReturn::Finish)
}

/// # Panics
/// Panics if user is unknown (only if message sent in channel)
pub async fn create_new_sticker_set<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    let set_title = message.text;

    // to generate random name for pack
    let charset = "abcg890hijklmJKxyzAnopqrstuvwBefCDEFGHIQRSTUVWXYZ1237LMNOP45d6";
    let set_name: Box<str> = String::into_boxed_str(generate(27, charset));

    let steal_sticker_set_name: Box<str> = if let Some(sssn) = fsm
        .get_value("steal_sticker_set_name")
        .await
        .map_err(Into::into)?
        .expect("Sticker set name for sticker set you want steal should be set")
    {
        sssn
    } else {
        error!("Error occurded while parsing name of this sticker pack.");

        bot.send(SendMessage::new(
            message.chat.id(),
            "Error occurded while parsing name of this sticker pack.",
        ))
        .await?;

        return Ok(EventReturn::Cancel);
    };

    // finish is not at the end because if an error occurs, the state will not be cleared
    fsm.finish().await.map_err(Into::into)?;

    let steal_stickers_from_sticker_set = bot
        .send(GetStickerSet::new(steal_sticker_set_name.as_ref()))
        .await?
        .stickers;

    let bot_user = bot
        .send(GetMe::new())
        .await?
        .username
        .expect("bot without username :/");

    let user = message.from.expect("error while parse user");
    let set_name = format!("{}_by_{}", set_name, bot_user);

    bot.send(SendMessage::new(
        message.chat.id(),
        format!("Creating sticker pack with name `{}` for you..", set_title),
    ))
    .await?;

    bot.send(CreateNewStickerSet::new(
        user.id,
        &set_name,
        set_title.as_ref(),
        steal_stickers_from_sticker_set.into_iter().map(|sticker| {
            let sticker_is = InputSticker::new(
                InputFile::id(sticker.file_id.as_ref()),
                if sticker.is_animated {
                    "animated"
                } else if sticker.is_video {
                    "video"
                } else {
                    "static"
                },
            );

            sticker_is.emoji_list(sticker.clone().emoji)
        }),
    ))
    .await?;

    let link = format!("t.me/addstickers/{}", set_name);
    let steal_link = format!("t.me/addstickers/{}", steal_sticker_set_name);

    let send_sticker_set = format!(
        "You successfully stole the {steal_ss_url}!\nThen you have your sticker set: {new_ss_url}.",
        steal_ss_url = html_text_link(steal_sticker_set_name, &steal_link),
        new_ss_url = html_text_link(&set_title, link)
    );

    bot.send(SendMessage::new(message.chat.id(), send_sticker_set).parse_mode(ParseMode::HTML))
        .await?;

    // dev
    debug!("sticker set name:\n{set_name}\nsticker set title:\n{set_title}");

    Ok(EventReturn::Finish)
}
