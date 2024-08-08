use telers::{
    enums::ParseMode,
    event::{telegram::HandlerResult, EventReturn},
    fsm::{Context, Storage},
    methods::SendMessage,
    types::{Message, MessageSticker, MessageText},
    utils::text::{Formatter as _, HTMLFormatter},
    Bot,
};
use tracing::debug;

use crate::states;
use states::State;

pub async fn start_handler(bot: Bot, message: Message) -> HandlerResult {
    bot.send(SendMessage::new(
        message.chat().id(),
        "
    Hello! This is bot to steal stickers.\n\
    List of commands you can use: 
    /help - Show help message 
    /steal - Steal sticker pack
    /my_stickers - Your current sticker packs you was steal
        ",
    ))
    .await?;

    Ok(EventReturn::Finish)
}

pub async fn steal_handler<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    bot.send(SendMessage::new(
        message.chat.id(),
        "Send me a sticker and I will steal the sticker pack that the sent sticker belongs to!",
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
    fsm.set_state(State::NewStickerSetName)
        .await
        .map_err(Into::into)?;

    // dev
    debug!("dev: Name of sent sticker pack: {set_name:?}");

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
        "Oh my gah! Please, send me sticker and I will create new the sticker pack that the sent sticker belongs to.",
    ))
    .await?;

    Ok(EventReturn::Finish)
}

pub async fn get_new_sticker_set_name<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    let name = message.text.clone();

    fsm.set_value("new_sticker_set_name", name)
        .await
        .map_err(Into::into)?;

    fsm.set_state(State::NewStickerSetTitle)
        .await
        .map_err(Into::into)?;

    bot.send(SendMessage::new(
        message.chat.id(),
        "Almost done! Now write a title for your new sticker pack:",
    ))
    .await?;

    Ok(EventReturn::Finish)
}

pub async fn get_new_sticker_set_title<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    let set_title = message.text.clone();

    fsm.set_value("new_sticker_set_title", set_title.clone())
        .await
        .map_err(Into::into)?;

    // dev
    let set_name: Box<str> = fsm
        .get_value("new_sticker_set_name")
        .await
        .map_err(Into::into)?
        .expect("Sticker set name for new sticker set should be set");

    // dev
    debug!("dev: Your info about new sticker pack:\nTitle: {set_title:?}\nName: {set_name:?}");

    Ok(EventReturn::Finish)
}
