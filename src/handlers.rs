use telers::{
    event::{telegram::HandlerResult, EventReturn},
    fsm::{Context, Storage},
    methods::SendMessage,
    types::{Message, MessageSticker, MessageText},
    Bot,
};

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
pub async fn sticker_handler<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    bot.send(SendMessage::new(
        message.chat.id(),
        "Send me a sticker or set and I will make a new set of stickers based on it",
    ))
    .await?;

    fsm.set_state(State::Sticker).await.map_err(Into::into)?;

    Ok(EventReturn::Finish)
}
pub async fn steal_handler<S: Storage>(
    bot: Bot,
    message: MessageSticker,
    fsm: Context<S>,
) -> HandlerResult {
    let set_name = message.clone().sticker.set_name.unwrap();

    fsm.set_value("sticker", message.sticker)
        .await
        .map_err(Into::into)?;

    bot.send(SendMessage::new(message.chat.id(), set_name))
        .await?;

    Ok(EventReturn::Finish)
}
pub async fn wrong_content_type(bot: Bot, message: Message) -> HandlerResult {
    bot.send(SendMessage::new(
        message.chat().id(),
        "Oh may gah! Please, send me sticker, I don't know how to handle non-stickers :(",
    ))
    .await?;

    Ok(EventReturn::Finish)
}
